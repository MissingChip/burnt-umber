interface ColorElement {
    ortho?: boolean;
    meshes: THREE.Mesh[];
    pick_meshes: THREE.Mesh[];

    on_input_change(pos: THREE.Vector3): void;
}
