/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::itest;
use godot::builtin::NodePath;
use godot::engine::{global, node, Node, Node3D, NodeExt, PackedScene, SceneTree};
use godot::log::godot_print;
use godot::obj::Share;

pub fn run() -> bool {
    let mut ok = true;
    ok &= node_print();
    ok &= node_get_node();
    //ok &= node_scene_tree();
    ok
}

// TODO move to other test
#[itest]
fn node_print() {
    godot_print!("Test print, bool={} and int={}", true, 32);
}


#[itest]
fn node_get_node() {
    let mut child = Node3D::new_alloc();
    child.set_name("child".into());
    let child_id = child.instance_id();

    let mut parent = Node3D::new_alloc();
    parent.set_name("parent".into());
    parent.add_child(
        child.share().upcast(),
        false,
        node::InternalMode::INTERNAL_MODE_DISABLED,
    );

    let mut grandparent = Node::new_alloc();
    grandparent.set_name("grandparent".into());
    grandparent.add_child(
        parent.share().upcast(),
        false,
        node::InternalMode::INTERNAL_MODE_DISABLED,
    );

    // Directly on Gd<T>
    let found = grandparent.get_node_as::<Node3D>(NodePath::from("parent/child"));
    assert_eq!(found.instance_id(), child_id);

    // Deref via &T
    let found = grandparent.get_node_as::<Node3D>(NodePath::from("parent/child"));
    assert_eq!(found.instance_id(), child_id);

    grandparent.free();
}

/*
#[itest]
fn node_scene_tree() {
    let mut child = Node::new_alloc();
    child.set_name("kid".into());

    let mut parent = Node::new_alloc();
    parent.set_name("parent".into());
    parent.add_child(
        child.share(),
        false,
        node::InternalMode::INTERNAL_MODE_DISABLED,
    );

    let mut scene = PackedScene::new();
    let err = scene.pack(parent.share());
    assert_eq!(err, global::Error::OK);

    let mut tree = SceneTree::new_alloc();
    let err = tree.change_scene_to_packed(scene);
    assert_eq!(err, global::Error::OK);

    // Note: parent + child are not owned by PackedScene, thus need to be freed
    // (verified by porting this very test to GDScript)
    tree.free();
    parent.free();
    child.free();
}
*/