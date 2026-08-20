Interactive Viewers and Animation
=================================

Runtime Selection
-----------------

``Viewer.render()`` selects its backend from the Python runtime:

* Jupyter and Google Colab use an inline WebAssembly canvas.
* Plain Python scripts and terminal IPython use a native GUI window.

.. code-block:: python

   from cosmol_viewer import Viewer

   print(Viewer.get_environment())
   viewer = Viewer.render(scene, width=800, height=500)

The returned viewer must remain referenced while it is being updated.

Interaction Controls
--------------------

Scene settings can keep drag rotation while disabling zoom, or automatically
orbit the molecule around the current camera-relative horizontal axis:

.. code-block:: python

   scene.set_zoom_disabled(True)
   scene.set_auto_rotate(True, speed=20.0)

Streaming Updates
-----------------

Use IDs to replace content in a scene, then send the new scene to an existing
viewer:

.. code-block:: python

   scene.replace_shape("molecule", next_molecule)
   viewer.update(scene)

``update()`` is intended for live or streaming data where frames are not known
in advance.

Animation Playback
------------------

Use :class:`~cosmol_viewer.Animation` when all frames are available before
playback:

.. code-block:: python

   from cosmol_viewer import Animation, Scene, Viewer

   animation = Animation(interval=0.05, loops=-1, interpolate=False)
   for molecule in molecules:
       frame = Scene()
       frame.add_shape(molecule)
       animation.add_frame(frame)

   Viewer.play(animation, width=800, height=500)

``interval`` is measured in seconds. ``loops=-1`` repeats indefinitely.
``interpolate=True`` interpolates compatible scene frames. A static scene can
be attached with ``set_static_scene()`` for geometry shared by every frame.
