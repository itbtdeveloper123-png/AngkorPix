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

import tempfile
import json
import urllib.request
import urllib.error
import io

API_KEYS = [
    "i1cmRDff152isKk3okV2qQ2v",  # key 01
    "1bHFjXYqWNtJzt7KkUmxoDuo",  # key 02
    "cfZjHpfpWorht5P7W18FWhnL",  # key 03
]

STATE_FILE = os.path.join(tempfile.gettempdir(), "angkorpix_rmbg_api_state.json")

def load_api_state():
    try:
        if os.path.exists(STATE_FILE):
            with open(STATE_FILE, "r", encoding="utf-8") as f:
                return json.load(f)
    except Exception:
        pass
    return {"current_index": 0, "exhausted": []}

def save_api_state(state):
    try:
        with open(STATE_FILE, "w", encoding="utf-8") as f:
            json.dump(state, f)
    except Exception:
        pass

def try_remove_bg_api(input_path, output_path, explicit_key=None):
    """
    Calls remove.bg API to remove background with multi-key automatic failover.
    Switches to the next key when current key runs out of quota/credits.
    """
    keys = [explicit_key] if explicit_key else list(API_KEYS)
    state = load_api_state()
    start_idx = state.get("current_index", 0) % len(keys)
    exhausted = set(state.get("exhausted", []))

    try:
        orig_img = Image.open(input_path)
        orig_w, orig_h = orig_img.size
        buf = io.BytesIO()
        orig_img.convert("RGBA").save(buf, format="PNG")
        img_bytes = buf.getvalue()
    except Exception as e:
        sys.stderr.write(f"[remove.bg] Failed to read input image: {e}\n")
        return False

    boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW"
    body = (
        f"--{boundary}\r\n"
        'Content-Disposition: form-data; name="size"\r\n\r\nauto\r\n'
        f"--{boundary}\r\n"
        'Content-Disposition: form-data; name="image_file"; filename="input.png"\r\n'
        'Content-Type: image/png\r\n\r\n'
    ).encode("utf-8") + img_bytes + f"\r\n--{boundary}--\r\n".encode("utf-8")

    ordered_indices = [(start_idx + i) % len(keys) for i in range(len(keys))]

    for idx in ordered_indices:
        api_key = keys[idx]
        sys.stderr.write(f"[remove.bg] Attempting background removal using API Key #{idx + 1} ({api_key[:6]}...{api_key[-4:]})...\n")

        req = urllib.request.Request(
            "https://api.remove.bg/v1.0/removebg",
            data=body,
            headers={
                "X-Api-Key": api_key,
                "Content-Type": f"multipart/form-data; boundary={boundary}",
            },
        )

        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                res_data = resp.read()
                out_img = Image.open(io.BytesIO(res_data))
                if out_img.mode != "RGBA":
                    out_img = out_img.convert("RGBA")

                # Guarantee exact matching dimensions with original image
                if out_img.size != (orig_w, orig_h):
                    out_img = out_img.resize((orig_w, orig_h), Image.Resampling.LANCZOS)

                out_img.save(output_path, format="PNG")
                sys.stderr.write(f"[remove.bg] SUCCESS: Background removed using Key #{idx + 1}!\n")

                state["current_index"] = idx
                save_api_state(state)
                return True

        except urllib.error.HTTPError as e:
            err_body = ""
            try:
                err_body = e.read().decode("utf-8", errors="ignore")
            except Exception:
                pass
            sys.stderr.write(f"[remove.bg] Key #{idx + 1} returned HTTP {e.code}: {err_body}\n")

            if e.code in (402, 403, 429) or "insufficient_credits" in err_body:
                sys.stderr.write(f"[remove.bg] Key #{idx + 1} quota/credits exhausted! Switching to next key...\n")
                exhausted.add(idx)
                state["exhausted"] = list(exhausted)
                state["current_index"] = (idx + 1) % len(keys)
                save_api_state(state)
                continue
            else:
                continue

        except Exception as e:
            sys.stderr.write(f"[remove.bg] Network/connection error on Key #{idx + 1}: {e}\n")
            continue

    sys.stderr.write("[remove.bg] All API keys in the pool were attempted and failed or exhausted.\n")
    return False

def main():
    parser = argparse.ArgumentParser(description="AngkorPix AI Background Removal")
    parser.add_argument("--input", required=True, help="Input RGBA or RGB image path")
    parser.add_argument("--output", required=True, help="Output single-channel grayscale mask path")
    parser.add_argument("--model", default=None, help="Explicit path to ONNX model")
    parser.add_argument("--mode", default="auto", choices=["auto", "standard", "glass", "tree", "cloud", "plastic", "portrait", "hair", "api"],
                        help="Cutout mode: auto, standard (solid subjects), glass, tree, cloud, plastic, or api")
    parser.add_argument("--api-key", default=None, help="Optional explicit remove.bg API key")
    parser.add_argument("--use-api", action="store_true", help="Force use remove.bg API")
    args = parser.parse_args()

    # If API mode requested, run via remove.bg multi-key pool
    if args.mode == "api" or args.use_api:
        sys.stderr.write("[AngkorPix] Using remove.bg Cloud API with multi-key failover...\n")
        if try_remove_bg_api(args.input, args.output, explicit_key=args.api_key):
            print("SUCCESS_API")
            sys.exit(0)
        else:
            sys.stderr.write("[AngkorPix] remove.bg API failed or keys exhausted. Attempting local model fallback...\n")

    model_path = find_model(args.model)
    if not model_path:
        # If no local model is found, automatically attempt remove.bg Cloud API!
        sys.stderr.write("[AngkorPix] Local ONNX model not found. Attempting remove.bg Cloud API with multi-key failover...\n")
        if try_remove_bg_api(args.input, args.output, explicit_key=args.api_key):
            print("SUCCESS_API")
            sys.exit(0)
        sys.stderr.write("ERROR: No ONNX model found and remove.bg API failed. Please place rmbg_model.onnx in the models/ directory or verify internet connection.\n")
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
    active_providers = sess.get_providers()
    active_provider = active_providers[0] if active_providers else "Unknown"
    sys.stderr.write(f"[AngkorPix AI] Active Execution Provider: {active_provider}\n")
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
    elif args.mode in ["portrait", "hair"]:
        # Professional Portrait & Hair Matting
        # Decontaminate background color fringes and preserve thin hair strands
        mask_clean = np.clip((mask_arr - 10.0) / 235.0, 0.0, 1.0)
        orig_f32 = orig_np / 255.0

        # Identify fine hair transition boundary (soft alpha region)
        hair_boundary = (mask_clean > 0.05) & (mask_clean < 0.92)

        # Color decontamination: neutralize background spill in hair boundary
        decontam_rgb = orig_f32.copy()
        if np.any(hair_boundary):
            fg_core = mask_clean > 0.85
            if np.any(fg_core):
                fg_mean = np.mean(orig_f32[fg_core], axis=0)
                blend_factor = (1.0 - mask_clean[hair_boundary, np.newaxis]) * 0.45
                decontam_rgb[hair_boundary] = (
                    decontam_rgb[hair_boundary] * (1.0 - blend_factor) + 
                    fg_mean * blend_factor
                )

        hair_alpha = np.clip(mask_clean ** 0.92, 0.0, 1.0)
        rgba = np.dstack([decontam_rgb, hair_alpha])
        rgba_u8 = (np.clip(rgba, 0.0, 1.0) * 255.0).astype(np.uint8)
        out_img = Image.fromarray(rgba_u8)
        out_img.save(args.output, format="PNG")
        print(f"SUCCESS_HAIR: {model_path}")
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
