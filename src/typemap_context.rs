use std::any::TypeId;
use std::marker::PhantomData;

use typemap::TypeMap;

use super::Context;

struct TypeMapKey<T: 'static>(PhantomData<T>);

impl<T> typemap::Key for TypeMapKey<T> {
    type Value = T;
}

/// A [`Context`] implementation based on [`typemap::TypeMap`].
pub struct DefaultContext {
    layers: Vec<Layer>,
}

impl Default for DefaultContext {
    fn default() -> Self {
        DefaultContext {
            layers: vec![Layer { type_id: TypeId::of::<()>(), map: TypeMap::custom() }],
        }
    }
}

impl Context for DefaultContext {
    type Scope = Scope;

    fn start_scope<T: 'static>(&mut self) -> Scope {
        let type_id = TypeId::of::<T>();
        let index = self.layers.len();

        self.layers.push(Layer { type_id, map: TypeMap::custom() });

        Scope { type_id, index }
    }

    fn end_scope(&mut self, scope: Scope) {
        let layer = self.layers.pop().expect("Ending scope of empty layout");
        debug_assert_eq!(scope.type_id, layer.type_id, "Scope mismatch");
        debug_assert_eq!(scope.index, self.layers.len(), "Scope mismatch");
    }

    fn nth_last_scope(&self, n: usize) -> Option<TypeId> {
        self.layers.get(self.layers.len() - n - 1).map(|layer| layer.type_id)
    }

    fn get<T>(&self, scope: TypeId) -> Option<&T>
    where
        T: 'static,
    {
        let layer = self.layers.iter().rev().find(|layer| layer.type_id == scope)?;
        layer.map.get::<TypeMapKey<T>>()
    }

    #[inline]
    fn get_each<T>(&self) -> Box<dyn Iterator<Item = &T> + '_>
    where
        T: 'static,
    {
        Box::new(self.layers.iter().rev().filter_map(|layer| layer.map.get::<TypeMapKey<T>>()))
    }

    fn get_mut<T, F>(&mut self, scope: TypeId, default: F) -> &mut T
    where
        F: FnOnce() -> T,
        T: 'static,
    {
        let layer = match self.layers.iter_mut().rev().find(|layer| layer.type_id == scope) {
            Some(layer) => layer,
            None => panic!("Attempt to fetch from scope {:?} which is not in the stack", scope,),
        };
        layer.map.entry::<TypeMapKey<T>>().or_insert_with(default)
    }
}

struct Layer {
    type_id: TypeId,
    map:     TypeMap,
}

/// Return value for [`DefaultContext::start_scope`].
pub struct Scope {
    type_id: TypeId,
    index:   usize,
}
