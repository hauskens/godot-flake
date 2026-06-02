@tool
extends EditorScenePostImport


func _post_import(scene: Node):
	_add_collision(scene)
	return scene


func _add_collision(root: Node):
	for child in root.get_children():
		if child is MeshInstance3D:
			var shape: ConvexPolygonShape3D = child.mesh.create_convex_shape()
			var path := "res://flipper_collision.tres"
			var col := CollisionShape3D.new()
			ResourceSaver.save(shape, path)
