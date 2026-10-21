#!/usr/bin/env python3
"""
AI Background Removal inference runner for AngkorPix.
Supports rmbg_model.onnx (Bria AI RMBG-1.4 / RMBG-2.0 / BiRefNet) as well as u2net.onnx.
"""

import os
import sys
import argparse
import numpy as np
from PIL import Image

def find_model(explicit_path=None):
    if explicit_path and os.path.exists(explicit_path):
        return explicit_path

    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(script_dir)

    candidates = [
        os.path.join(repo_root, "models", "rmbg_model.onnx"),
        os.path.join(repo_root, "models", "u2net.onnx"),
        os.path.join("models", "rmbg_model.onnx"),
        os.path.join("models", "u2net.onnx"),
    ]
    # Check any .onnx in models/
    for base in [os.path.join(repo_root, "models"), "models"]:
        if os.path.isdir(base):
            for f in os.listdir(base):
                if f.lower().endswith(".onnx"):
                    candidates.append(os.path.join(base, f))

    # Known system fallback
    candidates.append(r"E:\09-coding\dist\app\_internal\rembg\models\u2net.onnx")

    for c in candidates:
        if os.path.exists(c):
            return c
    return None

def main():
    parser = argparse.ArgumentParser(description="AngkorPix AI Background Removal")
    parser.add_argument("--input", required=True, help="Input RGBA or RGB image path")
    parser.add_argument("--output", required=True, help="Output single-channel grayscale mask path")
    parser.add_argument("--model", default=None, help="Explicit path to ONNX model")
    parser.add_argument("--mode", default="auto", choices=["auto", "standard", "glass", "tree", "cloud", "plastic"],
                        help="Cutout mode: auto, standard (solid subjects), glass, tree, cloud, or plastic")
    args = parser.parse_args()

    model_path = find_model(args.model)
    if not model_path:
        sys.stderr.write("ERROR: No ONNX model found. Please place rmbg_model.onnx in the models/ directory.\n")
        sys.exit(1)

    try:
        import onnxruntime as ort
    except ImportError:
        sys.stderr.write("ERROR: onnxruntime is not installed in Python environment.\n")
        sys.exit(2)

    sess_options = ort.SessionOptions()
    # Optimize memory usage to prevent GatherND bad allocation on large models (BiRefNet/RMBG-2.0)
    sess_options.enable_mem_pattern = False
    sess_options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
    sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL

    available_providers = ort.get_available_providers()
    providers = []
    if "DmlExecutionProvider" in available_providers:
        providers.append("DmlExecutionProvider")
    if "CUDAExecutionProvider" in available_providers:
        providers.append("CUDAExecutionProvider")
    providers.append("CPUExecutionProvider")

    sess = ort.InferenceSession(model_path, sess_options=sess_options, providers=providers)
    inp = sess.get_inputs()[0]
    
    # Determine target resolution from model input tensor
    model_h = inp.shape[2] if isinstance(inp.shape[2], int) and inp.shape[2] > 0 else 1024
    model_w = inp.shape[3] if isinstance(inp.shape[3], int) and inp.shape[3] > 0 else 1024

    img = Image.open(args.input).convert("RGB")
    orig_w, orig_h = img.size

    resized = img.resize((model_w, model_h), Image.Resampling.BILINEAR)
    arr = np.array(resized).astype(np.float32) / 255.0

    # Normalization: BiRefNet and U2Net use ImageNet mean/std, RMBG-1.4 uses 0.5/1.0
    is_rmbg_14 = "rmbg-1.4" in model_path.lower()
    if is_rmbg_14:
        mean = np.array([0.5, 0.5, 0.5], dtype=np.float32)
        std = np.array([1.0, 1.0, 1.0], dtype=np.float32)
    else:
        mean = np.array([0.485, 0.456, 0.406], dtype=np.float32)
        std = np.array([0.229, 0.224, 0.225], dtype=np.float32)

    norm = ((arr - mean) / std).transpose(2, 0, 1)[np.newaxis, ...]

    try:
        preds = sess.run(None, {inp.name: norm.astype(np.float32)})
    except Exception as e:
        # Fallback to minimal-arena execution if initial allocation fails
        sys.stderr.write(f"Inference error ({e}), retrying with minimal memory arena...\n")
        sess_options.enable_cpu_mem_arena = False
        sess_fallback = ort.InferenceSession(model_path, sess_options=sess_options, providers=["CPUExecutionProvider"])
        preds = sess_fallback.run(None, {inp.name: norm.astype(np.float32)})
    pred = preds[0]
    while pred.ndim > 2:
        pred = pred[0]

    # Convert logits to probability via sigmoid if needed (BiRefNet, RMBG-2.0)
    if pred.min() < 0.0 or pred.max() > 1.0:
        pred = 1.0 / (1.0 + np.exp(-pred))

    mask_u8 = (pred * 255.0).clip(0, 255).astype(np.uint8)
    mask_img = Image.fromarray(mask_u8)
    mask_final = mask_img.resize((orig_w, orig_h), Image.Resampling.BILINEAR)
    mask_arr = np.array(mask_final).astype(np.float32)

    # Clean up background remnants (e.g. checkerboard / white matte trapped in holes/gaps)
    orig_np = np.array(img).astype(np.float32)
    h, w, _ = orig_np.shape
    bw = min(5, w, h)
    borders = np.concatenate([
        orig_np[:bw, :, :].reshape(-1, 3),
        orig_np[-bw:, :, :].reshape(-1, 3),
        orig_np[:, :bw, :].reshape(-1, 3),
        orig_np[:, -bw:, :].reshape(-1, 3)
    ], axis=0)

    b_mean = np.mean(borders, axis=0)
    is_border_bright_neutral = (
        np.abs(b_mean[0] - b_mean[1]) < 14 and 
        np.abs(b_mean[1] - b_mean[2]) < 14 and 
        b_mean.mean() > 185
    )

    is_bg_match = (
        (orig_np[:, :, 0] > 185) & (orig_np[:, :, 1] > 185) & (orig_np[:, :, 2] > 185) &
        (np.abs(orig_np[:, :, 0] - orig_np[:, :, 1]) < 18) &
        (np.abs(orig_np[:, :, 1] - orig_np[:, :, 2]) < 18)
    )

    # Detect if subject is a transparent / glass object
    raw_fg = mask_arr > 120
    raw_fg_count = np.sum(raw_fg)
    bg_inside_fg_count = np.sum(raw_fg & is_bg_match)
    is_glass_detected = (
        args.mode == "glass" or 
        (args.mode == "auto" and is_border_bright_neutral and raw_fg_count > 100 and (bg_inside_fg_count / raw_fg_count) > 0.30)
    )

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)

    if (is_glass_detected or args.mode == "glass") and args.mode not in ["standard", "tree", "cloud", "plastic"]:
        # Smart Glass Cutout: extract transparency, highlights, shadows, and decontaminate background
        mask_clean = np.clip((mask_arr - 15.0) / 210.0, 0.0, 1.0)
        orig_f32 = orig_np / 255.0
        lum = 0.299 * orig_f32[:, :, 0] + 0.587 * orig_f32[:, :, 1] + 0.114 * orig_f32[:, :, 2]
        darkness = np.clip((0.85 - lum) / 0.85, 0.0, 1.0)

        # Specular reflection highlights (L > 0.76)
        hl = np.clip((lum - 0.76) / 0.24, 0.0, 1.0)

        # Alpha channel: opaque along contours and highlights, optical transparency on glass body
        # Base glass transparency is 0.28 so the bottle is always clearly visible on any background!
        alpha = np.clip(0.28 + darkness * 0.68 + hl * 0.65, 0.25, 0.98) * mask_clean

        # Blend highlight (pure white) with glass body
        hl_3d = hl[:, :, np.newaxis]
        rgb_decontam = orig_f32 * (1.0 - hl_3d) + 1.0 * hl_3d

        rgba = np.dstack([rgb_decontam, alpha])
        rgba_u8 = (np.clip(rgba, 0.0, 1.0) * 255.0).astype(np.uint8)
        out_img = Image.fromarray(rgba_u8)
        out_img.save(args.output, format="PNG")
        print(f"SUCCESS_GLASS: {model_path}")
    elif args.mode == "tree":
        # Photoshop Blue Channel Masking technique for fine leaves & branches
        orig_f32 = orig_np / 255.0
        r, g, b = orig_f32[:, :, 0], orig_f32[:, :, 1], orig_f32[:, :, 2]
        sky_intensity = np.clip((b - 0.5 * r - 0.1) / 0.4, 0.0, 1.0)
        mask_clean = np.clip((mask_arr - 20.0) / 210.0, 0.0, 1.0)
        tree_alpha = np.clip(mask_clean * (1.0 - sky_intensity), 0.0, 1.0)
        decontam_rgb = orig_f32.copy()
        is_spill = (b > g) & (b > r) & (tree_alpha < 0.8)
        decontam_rgb[is_spill, 2] = np.maximum(r[is_spill], g[is_spill] * 0.8)
        rgba = np.dstack([decontam_rgb, tree_alpha])
        rgba_u8 = (np.clip(rgba, 0.0, 1.0) * 255.0).astype(np.uint8)
        out_img = Image.fromarray(rgba_u8)
        out_img.save(args.output, format="PNG")
        print(f"SUCCESS_TREE: {model_path}")
    elif args.mode == "cloud":
        # Photoshop Blend If / Color Range technique for soft clouds & smoke
        orig_f32 = orig_np / 255.0
        r, g, b = orig_f32[:, :, 0], orig_f32[:, :, 1], orig_f32[:, :, 2]
        cloud_density = np.clip((r + g - 1.2 * b + 0.2) / 0.7, 0.0, 1.0)
        lum = 0.299 * r + 0.587 * g + 0.114 * b
        mask_clean = np.clip((mask_arr - 20.0) / 210.0, 0.0, 1.0)
        cloud_alpha = np.clip(mask_clean * (cloud_density * 0.7 + lum * 0.3), 0.0, 1.0)
        rgba = np.dstack([orig_f32, cloud_alpha])
        rgba_u8 = (np.clip(rgba, 0.0, 1.0) * 255.0).astype(np.uint8)
        out_img = Image.fromarray(rgba_u8)
        out_img.save(args.output, format="PNG")
        print(f"SUCCESS_CLOUD: {model_path}")
    elif args.mode == "plastic":
        # Photoshop High Pass / Screen + Multiply technique for transparent plastic bags
        mask_clean = np.clip((mask_arr - 25.0) / 200.0, 0.0, 1.0)
        orig_f32 = orig_np / 255.0
        lum = 0.299 * orig_f32[:, :, 0] + 0.587 * orig_f32[:, :, 1] + 0.114 * orig_f32[:, :, 2]
        darkness = np.clip(1.0 - lum, 0.0, 1.0)
        hl = np.clip((lum - 0.88) / 0.12, 0.0, 1.0) ** 1.6
        alpha = np.clip(darkness * 2.0 + hl * 1.8 + 0.04, 0.0, 1.0) * mask_clean
        hl_sum = darkness + hl
        ratio = np.where(hl_sum > 0.001, hl / hl_sum, 0.5)[:, :, np.newaxis]
        rgb_decontam = orig_f32 * (1.0 - ratio) + 1.0 * ratio
        rgba = np.dstack([rgb_decontam, alpha])
        rgba_u8 = (np.clip(rgba, 0.0, 1.0) * 255.0).astype(np.uint8)
        out_img = Image.fromarray(rgba_u8)
        out_img.save(args.output, format="PNG")
        print(f"SUCCESS_PLASTIC: {model_path}")
    else:
        if is_border_bright_neutral:
            mask_arr[is_bg_match] = 0.0

        # Suppress weak edge noise / halos
        mask_arr[mask_arr < 20] = 0.0

        mask_cleaned = Image.fromarray(mask_arr.clip(0, 255).astype(np.uint8))
        mask_cleaned.save(args.output, format="PNG")
        print(f"SUCCESS: {model_path}")

if __name__ == "__main__":
    main()
