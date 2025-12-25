import json
import os
import shutil
import re

def organize_assets():
    # Paths
    base_dir = os.getcwd()
    source_dir = os.path.join(base_dir, "backgrounds")
    target_dir = os.path.join(base_dir, "assets", "textures")
    json_path = os.path.join(base_dir, "assets", "graphics_batch.json")

    # Create target directory
    if not os.path.exists(target_dir):
        os.makedirs(target_dir)
        print(f"Created directory: {target_dir}")

    # Load JSON
    try:
        with open(json_path, 'r') as f:
            data = json.load(f)
            prompts = data.get("image_prompts", [])
    except FileNotFoundError:
        print("Error: graphics_batch.json not found.")
        return

    # Map IDs to filenames
    # The generated filenames seem to be: {id}_{id}_{resolution}_{date}.png
    # But sometimes just {id}_{resolution}... let's be flexible.
    
    files = os.listdir(source_dir)
    print(f"Found {len(files)} files in source directory.")

    moved_count = 0
    
    for prompt in prompts:
        img_id = prompt["id"]
        # Look for a file that starts with this ID
        # Heuristic: the filename usually matches exactly, or repeats the ID.
        # e.g. "tenant_student" -> "tenant_student_tenant_student_512x512..."
        
        candidates = []
        for filename in files:
            if filename.startswith(img_id):
                candidates.append(filename)
        
        if not candidates:
            print(f"WARNING: No file found for ID: {img_id}")
            continue
            
        # If multiple, take the one that matches best? 
        # For now, just take the first one, or the one that repeats the ID if available (often getting better generation from batch tools)
        # Actually, let's just take the first one found.
        best_candidate = candidates[0]
        
        src_path = os.path.join(source_dir, best_candidate)
        dest_filename = f"{img_id}.png"
        dest_path = os.path.join(target_dir, dest_filename)
        
        # Copy instead of move for safety during dev
        shutil.copy2(src_path, dest_path)
        print(f"Copied: {best_candidate} -> {dest_filename}")
        moved_count += 1

    print(f"Successfully organized {moved_count} assets.")

if __name__ == "__main__":
    organize_assets()
