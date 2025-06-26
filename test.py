import cosviewer

viewer = cosviewer.CosViewer()
viewer.set_state("Hello, Rust & Python!")
print(viewer.get_state())  # "Hello, Rust & Python!"

viewer.view()