import bpy
import math

# Clear scene
bpy.ops.wm.read_factory_settings(use_empty=True)


def create_image_material(name, image_path):
    try:
        img = bpy.data.images.load(image_path)
    except:
        print(f"Could not load image: {image_path}")
        return None, 1.0, 1.0

    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    shader = nodes.new('ShaderNodeBsdfPrincipled')
    shader.inputs['Alpha'].default_value = 1.0
    
    tex = nodes.new('ShaderNodeTexImage')
    tex.image = img

    out = nodes.new('ShaderNodeOutputMaterial')

    links.new(tex.outputs['Color'], shader.inputs['Base Color'])
    links.new(tex.outputs['Alpha'], shader.inputs['Alpha'])
    links.new(shader.outputs['BSDF'], out.inputs['Surface'])
    
    mat.blend_method = 'BLEND'
    return mat, img.size[0], img.size[1]

def create_text_material(name, color):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    shader = nodes.new('ShaderNodeBsdfPrincipled')
    shader.inputs['Base Color'].default_value = color
    # Add some emission so it pops
    shader.inputs['Emission Color'].default_value = color
    shader.inputs['Emission Strength'].default_value = 0.5

    out = nodes.new('ShaderNodeOutputMaterial')
    links.new(shader.outputs['BSDF'], out.inputs['Surface'])
    
    mat.blend_method = 'BLEND'
    return mat

def keyframe_visibility(obj, start_frame, end_frame):
    # Hide initially
    obj.hide_render = True
    obj.hide_viewport = True
    obj.keyframe_insert(data_path="hide_render", frame=0)
    obj.keyframe_insert(data_path="hide_viewport", frame=0)

    # Show at start
    obj.hide_render = False
    obj.hide_viewport = False
    obj.keyframe_insert(data_path="hide_render", frame=start_frame)
    obj.keyframe_insert(data_path="hide_viewport", frame=start_frame)

    # Hide at end
    obj.hide_render = True
    obj.hide_viewport = True
    obj.keyframe_insert(data_path="hide_render", frame=end_frame)
    obj.keyframe_insert(data_path="hide_viewport", frame=end_frame)

def to_blender_coords(x, y, res_x, res_y):
    # Map 0,0 (top-left) to -W/2, H/2
    # Scale: 100px = 1 unit
    bx = (x - res_x/2) / 100.0
    by = (res_y/2 - y) / 100.0
    return bx, by
scene = bpy.context.scene
scene.render.resolution_x = 1920
scene.render.resolution_y = 1080
scene.render.fps = 60
import sys
argv = sys.argv
if "--" in argv:
    args = argv[argv.index("--") + 1:]
    if "--start" in args:
        scene.frame_start = int(args[args.index("--start") + 1])
    if "--end" in args:
        scene.frame_end = int(args[args.index("--end") + 1])
    if "--output" in args:
        scene.render.filepath = args[args.index("--output") + 1]
else:
    scene.frame_start = 0
    scene.frame_end = 600
scene.render.image_settings.file_format = 'PNG'
scene.render.image_settings.color_mode = 'RGBA'

# Camera setup
cam_data = bpy.data.cameras.new(name='Camera')
cam_obj = bpy.data.objects.new(name='Camera', object_data=cam_data)
scene.collection.objects.link(cam_obj)
scene.camera = cam_obj
cam_obj.location = (0, 0, 10)
cam_data.type = 'ORTHO'
cam_data.ortho_scale = 10.8

# Layer: Image_hook_0
mat, img_w, img_h = create_image_material('Mat_Image_hook_0', 'assets/background.png')
if mat:
    bpy.ops.mesh.primitive_plane_add(size=1)
    obj = bpy.context.active_object
    obj.name = 'Image_hook_0'
    obj.data.materials.append(mat)
    obj.scale.x = img_w / 100.0
    obj.scale.y = img_h / 100.0
    bx, by = to_blender_coords(0, 0, 1920, 1080)
    obj.location.x = bx
    obj.location.y = by
    obj.scale.x *= 1
    obj.scale.y *= 1
    keyframe_visibility(obj, 0, 180)

# Layer: Text_hook_1
bpy.ops.object.text_add()
obj = bpy.context.active_object
obj.name = 'Text_hook_1'
obj.data.body = 'The Digital Artisan'
try:
    fnt = bpy.data.fonts.load('assets/font.ttf')
    obj.data.font = fnt
except:
    pass
obj.data.size = 72 / 100.0
mat = create_text_material('Mat_Text_hook_1', (1, 1, 1, 1))
obj.data.materials.append(mat)
bx, by = to_blender_coords(960, 440, 1920, 1080)
obj.location.x = bx
obj.location.y = by
keyframe_visibility(obj, 0, 180)

# Layer: Text_hook_2
bpy.ops.object.text_add()
obj = bpy.context.active_object
obj.name = 'Text_hook_2'
obj.data.body = 'Building Videos with Rust'
try:
    fnt = bpy.data.fonts.load('assets/font.ttf')
    obj.data.font = fnt
except:
    pass
obj.data.size = 48 / 100.0
mat = create_text_material('Mat_Text_hook_2', (0.78431374, 0.78431374, 0.78431374, 1))
obj.data.materials.append(mat)
bx, by = to_blender_coords(960, 600, 1920, 1080)
obj.location.x = bx
obj.location.y = by
keyframe_visibility(obj, 0, 180)

# Layer: Image_bridge_0
mat, img_w, img_h = create_image_material('Mat_Image_bridge_0', 'assets/pipeline.png')
if mat:
    bpy.ops.mesh.primitive_plane_add(size=1)
    obj = bpy.context.active_object
    obj.name = 'Image_bridge_0'
    obj.data.materials.append(mat)
    obj.scale.x = img_w / 100.0
    obj.scale.y = img_h / 100.0
    bx, by = to_blender_coords(0, 0, 1920, 1080)
    obj.location.x = bx
    obj.location.y = by
    obj.scale.x *= 1
    obj.scale.y *= 1
    keyframe_visibility(obj, 180, 480)

# Layer: Text_bridge_1
bpy.ops.object.text_add()
obj = bpy.context.active_object
obj.name = 'Text_bridge_1'
obj.data.body = 'The Rendering Pipeline'
try:
    fnt = bpy.data.fonts.load('assets/font.ttf')
    obj.data.font = fnt
except:
    pass
obj.data.size = 56 / 100.0
mat = create_text_material('Mat_Text_bridge_1', (0.2901961, 0.61960787, 1, 1))
obj.data.materials.append(mat)
bx, by = to_blender_coords(960, 100, 1920, 1080)
obj.location.x = bx
obj.location.y = by
keyframe_visibility(obj, 180, 480)

# Layer: Image_payoff_0
mat, img_w, img_h = create_image_material('Mat_Image_payoff_0', 'assets/result.png')
if mat:
    bpy.ops.mesh.primitive_plane_add(size=1)
    obj = bpy.context.active_object
    obj.name = 'Image_payoff_0'
    obj.data.materials.append(mat)
    obj.scale.x = img_w / 100.0
    obj.scale.y = img_h / 100.0
    bx, by = to_blender_coords(0, 0, 1920, 1080)
    obj.location.x = bx
    obj.location.y = by
    obj.scale.x *= 0
    obj.scale.y *= 0
    keyframe_visibility(obj, 480, 600)

# Layer: Text_payoff_1
bpy.ops.object.text_add()
obj = bpy.context.active_object
obj.name = 'Text_payoff_1'
obj.data.body = 'Fast. Engaging. Trustworthy.'
try:
    fnt = bpy.data.fonts.load('assets/font.ttf')
    obj.data.font = fnt
except:
    pass
obj.data.size = 64 / 100.0
mat = create_text_material('Mat_Text_payoff_1', (0.14901961, 0.87058824, 0.5058824, 1))
obj.data.materials.append(mat)
bx, by = to_blender_coords(960, 900, 1920, 1080)
obj.location.x = bx
obj.location.y = by
keyframe_visibility(obj, 480, 600)

# Render animation
bpy.ops.render.render(animation=True)
