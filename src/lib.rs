#![feature(drain_filter)]

#[cfg(test)]
#[macro_use]
extern crate multimap;
#[cfg(not(test))]
extern crate multimap;

use multimap::MultiMap;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Price(pub u64);
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Size(pub u64);
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Meta(pub String);

impl From<u64> for Price {
    fn from(num: u64) -> Price {
        Price(num)
    }
}

impl From<u64> for Size {
    fn from(num: u64) -> Size {
        Size(num)
    }
}

impl From<String> for Meta {
    fn from(meta: String) -> Meta {
        Meta(meta)
    }
}

type ContainerData = MultiMap<Price, (Size, Meta)>;

#[derive(Default, Debug, Clone)]
pub struct Container {
    pub data: ContainerData,
}

impl Container {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_data(cd: ContainerData) -> Self {
        Self { data: cd }
    }

    pub fn add<P, S, M>(&mut self, price: P, size: S, meta: M)
    where
        P: Into<Price>,
        S: Into<Size>,
        M: Into<Meta>,
    {
        self.data.insert(price.into(), (size.into(), meta.into()));
    }

    pub fn process<P, S>(&mut self, price: P, size: S) -> Self
    where
        P: Into<Price>,
        S: Into<Size>,
    {
        let input_price = price.into();
        let input_size = size.into();

        let mut filtered_size = 0;
        let mut filtered_data = MultiMap::new();

        let mut filtered_by_price = self.data
            .iter_all_mut()
            .filter(|(price, _items)| price < &&input_price)
            .collect::<Vec<(&Price, &mut Vec<(Size, Meta)>)>>();

        filtered_by_price.sort_by(|l, r| l.0.cmp(r.0));

        filtered_by_price
            .iter_mut()
            .for_each(|(parent_price, items)| {
                let filtered_by_size = items
                    .drain_filter(|item| {
                        let item_size = (item.0).0;
                        let new_size = item_size + filtered_size;
                        if new_size <= input_size.0 {
                            filtered_size += item_size;
                            return true;
                        }
                        false
                    })
                    .collect::<Vec<(Size, Meta)>>();

                if !filtered_by_size.is_empty() {
                    filtered_data
                        .entry(**parent_price)
                        .or_insert_vec(filtered_by_size);
                }
            });

        Self::new_with_data(filtered_data)
    }

    pub fn without<P, S>(mut self, price: P, size: S) -> Self
    where
        P: Into<Price>,
        S: Into<Size>,
    {
        let price = price.into();
        let size = size.into();

        if let Some(vec) = self.data.get_vec_mut(&price) {
            if let Some(index) = vec.iter_mut().position(|i| i.0 == size) {
                vec.remove(index);
            }
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use ::*;

    #[test]
    fn test_process() {
        let expected = multimap!(
                Price(0) => (Size(0), Meta("".to_owned())),
                Price(0) => (Size(3), Meta("".to_owned())),
                Price(1) => (Size(4), Meta("".to_owned()))
            );
        let mut c = Container::new();
        c.add(0, 0, "".to_owned());
        c.add(0, 3, "".to_owned());
        c.add(1, 4, "".to_owned());
        c.add(1, 8, "".to_owned());
        c.add(3, 3, "".to_owned());
        c.add(4, 2, "".to_owned());

        let filtered = c.process(3, 7);

        assert_eq!(c.data.len(), 4);
        assert_eq!(filtered.data.len(), 2);

        println!("Expected: {:?}", expected);
        println!("Filtered: {:?}", filtered.data);

        // Check that filtered contains all the data that should be there.
        assert_eq!(expected.len(), filtered.data.len());

        for (price, items) in &expected {
            let f_vec = filtered.data.get_vec(&price);
            assert!(f_vec.is_some());
            let f_vec = f_vec.unwrap();
            assert_eq!(f_vec.len(), items.len());

            for item in items {
                f_vec.contains(&item);
            }

            // Check that not filtered data does not have anything from expected.
            if let Some(vec) = c.data.get_vec(&price) {
                for item in items {
                    assert!(!vec.contains(item));
                }
            }
        }
    }
}
