use leptos_reactive::{create_effect, create_signal, ReadSignal, Scope, ScopeDisposer};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, ops::IndexMut, rc::Rc};

/// Function that maps a `Vec` to another `Vec` via a map function. The mapped `Vec` is lazy
/// computed; its value will only be updated when requested. Modifications to the
/// input `Vec` are diffed using keys to prevent recomputing values that have not changed.
///
/// This function is the underlying utility behind `Keyed`.
///
/// # Params
/// * `list` - The list to be mapped. The list must be a [`ReadSignal`] (obtained from a [`Signal`])
///   and therefore reactive.
/// * `map_fn` - A closure that maps from the input type to the output type.
/// * `key_fn` - A closure that returns an _unique_ key to each entry.
///
///  _Credits: Based on implementation for [Sycamore](https://github.com/sycamore-rs/sycamore/blob/53735aab9ef72b98439b4d2eaeb85a97f7f32775/packages/sycamore-reactive/src/iter.rs),
/// which is in turned based on on the TypeScript implementation in <https://github.com/solidjs/solid>_
pub fn map_keyed<T, U, K>(
    cx: Scope,
    list: impl Fn() -> Vec<T> + 'static,
    map_fn: impl Fn(Scope, &T) -> U + 'static,
    key_fn: impl Fn(&T) -> K + 'static,
) -> ReadSignal<Vec<U>>
where
    T: PartialEq + Debug + Clone + 'static,
    K: Eq + Hash,
    U: PartialEq + Debug + Clone,
{
    // Previous state used for diffing.
    let mut mapped: Rc<RefCell<Vec<U>>> = Rc::new(RefCell::new(Vec::new()));
    let mut disposers: Vec<Option<ScopeDisposer>> = Vec::new();

    let (item_signal, set_item_signal) = create_signal(cx, Vec::new());

    // Diff and update signal each time list is updated.
    create_effect(cx, move |items| {
        let items: Vec<T> = items.unwrap_or_default();
        let new_items = list();
        let new_items_len = new_items.len();

        if new_items.is_empty() {
            // Fast path for removing all items.
            let disposers = std::mem::take(&mut disposers);
            leptos_reactive::queue_microtask(move || {
                for disposer in disposers {
                    disposer.unwrap().dispose();
                }
            });
            *mapped.borrow_mut() = Vec::new();
        } else if items.is_empty() {
            // Fast path for creating items when the existing list is empty.
            for new_item in new_items.iter() {
                let mut value: Option<U> = None;
                let new_disposer = cx.child_scope(|cx| {
                    value = Some(map_fn(cx, new_item));
                });
                mapped.borrow_mut().push(value.unwrap());
                disposers.push(Some(new_disposer));
            }
        } else {
            let mut temp = vec![None; new_items.len()];
            let mut temp_disposers: Vec<Option<ScopeDisposer>> =
                (0..new_items.len()).map(|_| None).collect();

            // Skip common prefix.
            let min_len = usize::min(items.len(), new_items.len());
            let start = items
                .iter()
                .zip(new_items.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(min_len);

            // Skip common suffix.
            let mut end = items.len();
            let mut new_end = new_items.len();
            #[allow(clippy::suspicious_operation_groupings)]
            // FIXME: make code clearer so that clippy won't complain
            while end > start && new_end > start && items[end - 1] == new_items[new_end - 1] {
                end -= 1;
                new_end -= 1;
                temp[new_end] = Some(mapped.borrow()[end].clone());
                temp_disposers[new_end] = disposers[end].take();
            }

            // 0) Prepare a map of indices in newItems. Scan backwards so we encounter them in
            // natural order.
            let mut new_indices = HashMap::with_capacity(new_end - start);

            // Indexes for new_indices_next are shifted by start because values at 0..start are
            // always None.
            let mut new_indices_next = vec![None; new_end - start];
            for j in (start..new_end).rev() {
                let item = &new_items[j];
                let i = new_indices.get(&key_fn(item));
                new_indices_next[j - start] = i.copied();
                new_indices.insert(key_fn(item), j);
            }

            // 1) Step through old items and see if they can be found in new set; if so, mark
            // them as moved.
            for i in start..end {
                let item = &items[i];
                if let Some(j) = new_indices.get(&key_fn(item)).copied() {
                    // Moved. j is index of item in new_items.
                    temp[j] = Some(mapped.borrow()[i].clone());
                    temp_disposers[j] = disposers[i].take();
                    new_indices_next[j - start].and_then(|j| new_indices.insert(key_fn(item), j));
                } else {
                    // Create new.
                    disposers[i].take().unwrap().dispose();
                }
            }

            // 2) Set all the new values, pulling from the moved array if copied, otherwise
            // entering the new value.
            for j in start..new_items.len() {
                if matches!(temp.get(j), Some(Some(_))) {
                    // Pull from moved array.
                    if j >= mapped.borrow().len() {
                        mapped.borrow_mut().push(temp[j].clone().unwrap());
                        disposers.push(temp_disposers[j].take());
                    } else {
                        *mapped.borrow_mut().index_mut(j) = temp[j].clone().unwrap();
                        disposers[j] = temp_disposers[j].take();
                    }
                } else {
                    // Create new value.
                    let mut tmp = None;
                    let new_item = &new_items[j];
                    let new_disposer = cx.child_scope(|cx| {
                        tmp = Some(map_fn(cx, new_item));
                    });
                    if mapped.borrow().len() > j {
                        mapped.borrow_mut()[j] = tmp.unwrap();
                        disposers[j] = Some(new_disposer);
                    } else {
                        mapped.borrow_mut().push(tmp.unwrap());
                        disposers.push(Some(new_disposer));
                    }
                }
            }
        }
        // 3) In case the new set is shorter than the old, set the length of the mapped array.
        mapped.borrow_mut().truncate(new_items_len);
        disposers.truncate(new_items_len);

        // 4) Update signal to trigger updates.
        set_item_signal({
            let mapped = Rc::clone(&mapped);
            move |n| *n = mapped.borrow().to_vec()
        });

        // 5) Return the new items, for use in next iteration
        new_items.to_vec()
    });

    item_signal
}
