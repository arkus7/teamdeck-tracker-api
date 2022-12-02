macro_rules! sort_by_enum {
    ($name:ident { $($variant:tt),+ }, $remote:ty) => {
        paste::paste! {
            #[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
            pub enum $name {
                $(
                    #[doc = "Sorts by `" $variant "` in ascending order."]
                    [<$variant Asc>],
                    #[doc = "Sorts by `" $variant "` in descending order."]
                    [<$variant Desc>],
                )+
            }

            impl From<$name> for teamdeck::api::sort_by::SortBy<$remote> {
                fn from(val: $name) -> Self {
                    match val {
                        $(
                            $name::[<$variant Asc>] => Self::Asc($remote::$variant),
                            $name::[<$variant Desc>] => Self::Desc($remote::$variant),
                        )+
                    }
                }
            }
        }
    };
}

pub(crate) use sort_by_enum;

sort_by_enum!(
    Test { Name, Email, Role },
    teamdeck::api::resources::ResourcesSortBy
);

#[cfg(test)]
mod test {
    use teamdeck::api::{resources::ResourcesSortBy, sort_by::SortBy};

    use super::*;

    sort_by_enum!(Test { Name, Email, Role }, ResourcesSortBy);

    #[test]
    fn test_name_asc() {
        let sort_by: SortBy<ResourcesSortBy> = Test::NameAsc.into();
        assert_eq!(sort_by, SortBy::Asc(ResourcesSortBy::Name));
    }

    #[test]
    fn test_role_desc() {
        let sort_by: SortBy<ResourcesSortBy> = Test::RoleDesc.into();
        assert_eq!(sort_by, SortBy::Desc(ResourcesSortBy::Role));
    }

    #[test]
    fn test_email_desc() {
        let sort_by: SortBy<ResourcesSortBy> = Test::EmailDesc.into();
        assert_eq!(sort_by, SortBy::Desc(ResourcesSortBy::Email));
    }

    #[test]
    fn test_full_import() {
        sort_by_enum!(
            Test { Name, Email, Role },
            teamdeck::api::resources::ResourcesSortBy
        );
    }
}
