import argparse
import json
import os
import random
import time
from datetime import datetime
import torch
from diffusers import DiffusionPipeline

def setup_pipeline(model_id):
    print(f"Loading model: {model_id}...")
    try:
        # Load the pipeline
        # Use bfloat16 for optimal performance on supported GPUs
        # Low cpu mem usage can help with loading
        pipe = DiffusionPipeline.from_pretrained(
            model_id,
            torch_dtype=torch.bfloat16,
            low_cpu_mem_usage=False,
        )
        pipe.to("cuda")
        return pipe
    except Exception as e:
        print(f"Error loading pipeline: {e}")
        return None

def generate_image(pipe, prompt, negative_prompt, width, height, steps, cfg, seed):
    if seed == -1:
        seed = random.randint(0, 2**32 - 1)
    
    generator = torch.Generator("cuda").manual_seed(seed)
    
    print(f"Generating: {prompt[:50]}...")
    print(f"Size: {width}x{height}, Seed: {seed}, Steps: {steps}, CFG: {cfg}")
    
    image = pipe(
        prompt=prompt,
        negative_prompt=negative_prompt,
        width=width,
        height=height,
        num_inference_steps=steps,
        guidance_scale=cfg,
        generator=generator
    ).images[0]
    
    return image

def main():
    parser = argparse.ArgumentParser(description="Z-Image-Turbo Batch Generator")
    parser.add_argument("--Prompt", type=str, help="Single prompt to generate")
    parser.add_argument("--PromptJsonFile", type=str, help="Path to JSON file with prompts")
    parser.add_argument("--Width", type=int, default=1024, help="Image width")
    parser.add_argument("--Height", type=int, default=1024, help="Image height")
    parser.add_argument("--NegativePrompt", type=str, default="bad hands, blurry, low quality", help="Negative prompt")
    parser.add_argument("--OutputPath", type=str, default="generated_image.png", help="Output path for single generation")
    parser.add_argument("--Steps", type=int, default=9, help="Inference steps")
    parser.add_argument("--CFG", type=float, default=0.0, help="Guidance scale")
    parser.add_argument("--Seed", type=int, default=-1, help="Random seed (-1 for random)")
    parser.add_argument("--Model", type=str, default="Tongyi-MAI/Z-Image-Turbo", help="Hugging Face model ID")
    parser.add_argument("--DelaySeconds", type=int, default=0, help="Delay between generations")
    
    args = parser.parse_args()
    
    # Initialize Pipeline
    pipe = setup_pipeline(args.Model)
    if not pipe:
        return

    # Create output directory for batch
    output_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "backgrounds")
    os.makedirs(output_dir, exist_ok=True)

    prompts_to_process = []

    if args.Prompt:
        prompts_to_process.append({
            "prompt": args.Prompt,
            "negative_prompt": args.NegativePrompt,
            "width": args.Width,
            "height": args.Height,
            "output_path": args.OutputPath, # Use specified path for single run
            "seed": args.Seed,
            "steps": args.Steps,
            "cfg": args.CFG
        })
    elif args.PromptJsonFile:
        if not os.path.exists(args.PromptJsonFile):
            print(f"Error: File not found {args.PromptJsonFile}")
            return
            
        with open(args.PromptJsonFile, 'r', encoding='utf-8') as f:
            data = json.load(f)
            
        items = data.get('image_prompts', []) if isinstance(data, dict) else data
        if not isinstance(items, list):
            items = []
            
        for item in items:
            # Logic to determine prompt text
            p_text = item.get('Prompt') or item.get('description') or item.get('title')
            if not p_text:
                continue
                
            # Logic for resolution
            w = item.get('Width', args.Width)
            h = item.get('Height', args.Height)
            
            # Logic for filename
            keyword = "image"
            if item.get('tags'):
                keyword = "_".join(item['tags']).lower()
            elif item.get('title'):
                keyword = item['title'].lower().replace(" ", "_")
            elif item.get('description'):
                keyword = "_".join(item['description'].split()[:2]).lower()
            
            # Sanitize keyword
            keyword = "".join(c for c in keyword if c.isalnum() or c in ('_', '-'))
            
            id_prefix = f"{item['id']}_" if item.get('id') else ""
            timestamp = datetime.now().strftime("%Y%m%d-%H%mmss")
            filename = f"{id_prefix}{keyword}_{w}x{h}_{timestamp}.png"
            
            prompts_to_process.append({
                "prompt": p_text,
                "negative_prompt": item.get('NegativePrompt', args.NegativePrompt),
                "width": w,
                "height": h,
                "output_path": os.path.join(output_dir, filename),
                "seed": item.get('Seed', args.Seed),
                "steps": item.get('Steps', args.Steps),
                "cfg": item.get('CFG', args.CFG)
            })
    else:
        print("Please provide --Prompt or --PromptJsonFile")
        return

    print(f"Found {len(prompts_to_process)} prompts to process.")
    
    for i, p in enumerate(prompts_to_process):
        print(f"\nProcessing {i+1}/{len(prompts_to_process)}")

        # Ensure dimensions are multiples of 16
        orig_w, orig_h = p['width'], p['height']
        p['width'] = (p['width'] // 16) * 16
        p['height'] = (p['height'] // 16) * 16
        
        if p['width'] != orig_w or p['height'] != orig_h:
            print(f"Warning: Adjusted dimensions from {orig_w}x{orig_h} to {p['width']}x{p['height']} (must be divisible by 16)")
        
        img = generate_image(
            pipe,
            p['prompt'],
            p['negative_prompt'],
            p['width'],
            p['height'],
            p['steps'],
            p['cfg'],
            p['seed']
        )
        
        img.save(p['output_path'])
        print(f"Saved to {p['output_path']}")
        
        if i < len(prompts_to_process) - 1 and args.DelaySeconds > 0:
            time.sleep(args.DelaySeconds)

if __name__ == "__main__":
    main()
