import * as THREE from 'three';

const unit = {
    x: new THREE.Vector3(1, 0, 0),
    y: new THREE.Vector3(0, 1, 0),
    z: new THREE.Vector3(0, 0, 1)
};

export interface CameraController {
    theta: number;
    phi: number;
    radius: number;
    // up: THREE.Vector3,
    lookAt: THREE.Vector3;

    on_move(delta: THREE.Vector3): void;
}

export function cameraController(camera: THREE.PerspectiveCamera): CameraController {
    const controller = {
        theta: 0,
        phi: 1,
        radius: 4,
        lookAt: new THREE.Vector3(0, 1, 0),

        on_move(delta: THREE.Vector3) {
            this.theta += delta.y * 0.01;
            this.phi += delta.x * 0.01;
            this.radius += delta.z * 0.04;
            this.radius = Math.max(this.radius, 0.1);
            const position = new THREE.Vector3(
                this.radius * Math.cos(this.theta) * Math.cos(this.phi),
                this.radius * Math.sin(this.theta),
                this.radius * Math.cos(this.theta) * Math.sin(this.phi)
            );
            // TODO: Fix up vector
            const up = new THREE.Vector3(
                -Math.cos(this.phi),
                Math.cos(this.theta - Math.PI / 4),
                -Math.sin(this.phi)
            );
            up.normalize();
            position.normalize().multiplyScalar(this.radius);
            position.add(this.lookAt);
            camera.up.copy(up);
            camera.position.copy(position);
            camera.lookAt(this.lookAt);
        }
    };
    controller.on_move(new THREE.Vector3(0, 0, 0));
    return controller;
}