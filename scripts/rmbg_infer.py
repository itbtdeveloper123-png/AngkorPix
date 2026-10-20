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

    sess = ort.InferenceSession(model_path, providers=["CPUExecutionProvider"])
    inp = sess.get_inputs()[0]
    
    # Determine target resolution from model input tensor
    model_h = inp.shape[2] if isinstance(inp.shape[2], int) and inp.shape[2] > 0 else 1024
    model_w = inp.shape[3] if isinstance(inp.shape[3], int) and inp.shape[3] > 0 else 1024

    img = Image.open(args.input).convert("RGB")
    orig_w, orig_h = img.size

    resized = img.resize((model_w, model_h), Image.Resampling.BILINEAR)
    arr = np.array(resized).astype(np.float32) / 255.0

    # Normalization: RMBG/BiRefNet usually uses [0.5, 0.5, 0.5], U2Net uses ImageNet mean/std
    is_u2net = "u2net" in model_path.lower()
    if is_u2net:
        mean = np.array([0.485, 0.456, 0.406], dtype=np.float32)
        std = np.array([0.229, 0.224, 0.225], dtype=np.float32)
    else:
        mean = np.array([0.5, 0.5, 0.5], dtype=np.float32)
        std = np.array([1.0, 1.0, 1.0], dtype=np.float32)

    norm = ((arr - mean) / std).transpose(2, 0, 1)[np.newaxis, ...]

    preds = sess.run(None, {inp.name: norm.astype(np.float32)})
    pred = preds[0]
    if pred.ndim == 4:
        pred = pred[0, 0]
    elif pred.ndim == 3:
        pred = pred[0]

    val_min = float(pred.min())
    val_max = float(pred.max())
    if val_max > val_min:
        pred = (pred - val_min) / (val_max - val_min)
    else:
        pred = np.zeros_like(pred)

    mask_u8 = (pred * 255.0).clip(0, 255).astype(np.uint8)
    mask_img = Image.fromarray(mask_u8)
    mask_final = mask_img.resize((orig_w, orig_h), Image.Resampling.BILINEAR)

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    mask_final.save(args.output, format="PNG")
    print(f"SUCCESS: {model_path}")

if __name__ == "__main__":
    main()
