from typing import Optional, Union, List

def parse_sdf(
    sdf: str,
    keep_h: bool = True,
    multimodel: bool = True,
    onemol: bool = False
) -> MoleculeData:
    """
    Parse an SDF string into molecule data.

    # Args
      - sdf: Path to an SD string containing SDF content.
      - keep_h: Whether to keep explicit hydrogen atoms (default: True).
      - multimodel: Whether to allow multiple models in one file (default: True).
      - onemol: Whether to merge multiple models into one molecule (default: False).

    # Returns
      - MoleculeData: Parsed molecule data object.

    # Example
    ```python
    from cosmol_viewer import parse_sdf
    mol = parse_sdf(open("./molecule.sdf", "r", encoding="utf-8").read())
    ```
    """
    ...

def parse_mmcif(
    mmcif: str
) -> ProteinData:
    """
    Parse an MMCIF string into protein data.

    # Args
      - mmcif: Path to an MMCIF string containing MMCIF content.

    # Returns
      - ProteinData: Parsed protein data object.

    # Example
    ```python
    from cosmol_viewer import parse_mmcif
    prot = parse_mmcif(open("./protein.cif", "r", encoding="utf-8").read())
    ```
    """
    ...

class Scene:
    """
    A 3D scene container for visualizing molecular or geometric shapes.

    This class allows adding, updating, and removing shapes in a 3D scene,
    as well as modifying scene-level properties like scale and background color.

    Supported shape types:
      - PySphere
      - PyStick
      - PyMolecules

    Shapes can be optionally identified with a string `id`,
    which allows updates and deletion.
    """

    def __init__(self) -> None:
        """
        Creates a new empty scene.

        # Example
        ```python
        scene = Scene()
        ```
        """
        ...

    def add_shape(self, shape: Union["Sphere", "Stick", "Molecules", "Protein"], id: Optional[str] = None) -> None:
        """
        Add a shape to the scene.

        # Args
          - shape: A shape instance (Sphere, Stick, Molecules, or Protein).
          - id: Optional string ID to associate with the shape.

        If the `id` is provided and a shape with the same ID exists,
        the new shape will replace it.

        # Example
        ```python
        scene.add_shape(sphere)
        scene.add_shape(stick, id="bond1")
        ```
        """
        ...

    def update_shape(self, id: str, shape: Union["Sphere", "Stick", "Molecules", "Protein"]) -> None:
        """
        Update an existing shape in the scene by its ID.

        # Args
          - id: ID of the shape to update.
          - shape: New shape object to replace the existing one.

        # Example
        ```python
        scene.update_shape("atom1", updated_sphere)
        ```
        """
        ...

    def delete_shape(self, id: str) -> None:
        """
        Remove a shape from the scene by its ID.

        # Args
          - id: ID of the shape to remove.

        # Example
        ```python
        scene.delete_shape("bond1")
        ```
        """
        ...

    def recenter(self, center: List[float]) -> None:
        """
        Recenter the scene at a given point.

        # Args
          - center: An XYZ array of 3 float values representing the new center.

        # Example
        ```python
        scene.recenter([0.0, 0.0, 0.0])
        ```
        """
        ...

    def scale(self, scale: float) -> None:
        """
        Set the global scale factor of the scene.

        This affects the visual size of all shapes uniformly.

        # Args
          - scale: A positive float scaling factor.

        # Example
        ```python
        scene.scale(1.5)
        ```
        """
        ...

    def set_background_color(self, background_color: List[float]) -> None:
        """
        Set the background color of the scene.

        # Args
          - background_color: An RGB array of 3 float values between 0.0 and 1.0.

        # Example
        ```python
        scene.set_background_color([1.0, 1.0, 1.0])  # white background
        ```
        """
        ...

    def use_black_background(self) -> None:
        """
        Set the background color of the scene to black.

        # Example
        ```python
        scene.use_black_background()
        ```
        """
        ...

class Viewer:
    """
    A viewer that renders 3D scenes in different runtime environments
    (e.g., Jupyter, Colab, or native GUI).

    The `Viewer` automatically selects a backend:
      - Jupyter/Colab → WebAssembly canvas (inline display)
      - Python script/terminal → native GUI window (if supported)

    Use `Viewer.render(scene)` to create and display a viewer instance.
    """

    @staticmethod
    def get_environment() -> str:
        """
        Get the current runtime environment.

        # Returns
          - str: One of "Jupyter", "Colab", "PlainScript", or "IPythonTerminal".

        # Example
        ```python
        env = Viewer.get_environment()
        print(env)  # e.g., "Jupyter"
        ```
        """
        ...

    @staticmethod
    def render(scene: "Scene", width: float = 800.0, height: float = 600.0) -> "Viewer":
        """
        Render a 3D scene.

        # Args
          - scene: The scene to render.
          - width: The viewport width in pixels (default: 800).
          - height: The viewport height in pixels (default: 600).

        # Returns
          - Viewer: The created viewer instance.

        # Example
        ```python
        from cosmol_viewer import Viewer, Scene, Sphere
        scene = Scene()
        scene.add_shape(Sphere([0, 0, 0], 1.0))
        viewer = Viewer.render(scene)
        ```
        """
        ...

    @staticmethod
    def play(
        frames: List["Scene"],
        interval: float,
        loops: int,
        width: float = 800.0,
        height: float = 600.0,
        smooth: bool = False
    ) -> "Viewer":
        """
        Play an animation of multiple frames.

        # Args
          - frames: List of Scene objects as animation frames.
          - interval: Frame interval in seconds.
          - loops: Number of loops to repeat (-1 for infinite).
          - width: The viewport width in pixels.
          - height: The viewport height in pixels.
          - smooth: Whether to smooth the animation by
            interpolating between frames.

        # Returns
          - Viewer: The created viewer instance.

        # Example
        ```python
        viewer = Viewer.play([scene1, scene2], interval=0.5, loops=3)
        ```
        """
        ...

    def update(self, scene: "Scene") -> None:
        """
        Update the viewer with a new scene.

        Works for both Web-based rendering (Jupyter/Colab) and native GUI windows.

        ⚠️ Note (Jupyter/Colab): Animation updates may be limited by
        notebook rendering capacity.

        # Args
          - scene: The updated scene.

        # Example
        ```python
        scene.add_shape(Sphere([1, 1, 1], 0.5))
        viewer.update(scene)
        ```
        """
        ...

    def save_image(self, path: str) -> None:
        """
        Save the current image to a file.

        # Args
          - path: File path for the saved image.

        # Example
        ```python
        viewer.save_image("output.png")
        ```
        """
        ...

class Sphere:
    """
    A sphere shape in the scene.

    # Args
      - center: [x, y, z] coordinates of the sphere center.
      - radius: Radius of the sphere.

    # Example
    ```python
    sphere = Sphere([0, 0, 0], 1.0).color([1, 0, 0])
    ```
    """

    def __init__(self, center: List[float], radius: float) -> None: ...
    def set_center(self, center: List[float]) -> "Sphere": ...
    def set_radius(self, radius: float) -> "Sphere": ...
    def color(self, color: List[float]) -> "Sphere": ...
    def color_rgba(self, color: List[float]) -> "Sphere": ...
    def opacity(self, opacity: float) -> "Sphere": ...


class Stick:
    """
    A cylindrical stick (or capsule) connecting two points.

    # Args
      - start: Starting point [x, y, z].
      - end: Ending point [x, y, z].
      - thickness: Stick radius.

    # Example
    ```python
    stick = Stick([0,0,0], [1,1,1], 0.1).opacity(0.5)
    ```
    """

    def __init__(self, start: List[float], end: List[float], thickness: float) -> None: ...
    def color(self, color: List[float]) -> "Stick": ...
    def color_rgba(self, color: List[float]) -> "Stick": ...
    def opacity(self, opacity: float) -> "Stick": ...
    def set_thickness(self, thickness: float) -> "Stick": ...
    def set_start(self, start: List[float]) -> "Stick": ...
    def set_end(self, end: List[float]) -> "Stick": ...


class Molecules:
    """
    A molecular shape object.

    # Example
    ```python
    mol = parse_sdf(open("molecule.sdf", "r", encoding="utf-8").read())
    molecules = Molecules(mol).centered().color([0,1,0])
    ```
    """

    def __init__(self, molecule_data: "MoleculeData") -> None: ...
    def get_center(self) -> List[float]: ...
    def centered(self) -> "Molecules": ...
    def color(self, color: List[float]) -> "Molecules": ...
    def color_rgba(self, color: List[float]) -> "Molecules": ...
    def opacity(self, opacity: float) -> "Molecules": ...
    def reset_color(self) -> "Molecules": ...

class MoleculeData:
    """
    Internal representation of molecule data returned by `parse_sdf`.
    """
    ...

class Protein:
    """
    A protein shape object.

    # Example
    ```python
    mmcif_data  = parse_mmcif(open("2AMD.cif", "r", encoding="utf-8").read())
    prot = Protein(mmcif_data).centered().color([0,1,0])
    ```
    """

    def __init__(self, mmcif_data: "ProteinData") -> None: ...
    def get_center(self) -> List[float]: ...
    def centered(self) -> "Protein": ...
    def color(self, color: List[float]) -> "Protein": ...
    def color_rgba(self, color: List[float]) -> "Protein": ...
    def opacity(self, opacity: float) -> "Protein": ...
    def reset_color(self) -> "Protein": ...

class ProteinData:
    """
    Internal representation of protein data returned by `parse_mmcif`.
    """
    ...
