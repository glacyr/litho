use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    let schema = r#"
    input ComplexInput {
        name: String
        owner: String
    }

    interface Pet {
        name: String!
    }

    type Cat implements Pet {
        name: String!
    }

    type Dog implements Pet {
        name: String!
        isHouseTrained(atOtherHomes: Boolean): Boolean!
    }

    union CatOrDog = Cat | Dog

    type Query {
        dog: Dog
        findDog(complex: ComplexInput): Dog
        booleanList(booleanListArg: [Boolean!]): Boolean
        outputCat(cat: Cat): Cat
        outputDogBang(dog: Dog!): Dog!
        outputListOfPets(pets: [Pet]): [Pet]
        outputCatOrDog(catOrDog: CatOrDog): CatOrDog
    }

    interface Bar {
        bar: Boolean!
    }

    type Subscription implements Bar {
        foo: Int!
        bar: Boolean!
    }
    "#;

    let query = r#"
    query takesBoolean($atOtherHomes: Boolean) {
        dog {
            isHouseTrained(atOtherHomes: $atOtherHomes)
        }
    }

    query takesComplexInput($complexInput: ComplexInput) {
        findDog(complex: $complexInput) {
            name
        }
    }

    query TakesListOfBooleanBang($booleans: [Boolean!]) {
        booleanList(booleanListArg: $booleans)
    }

    query takesCat($cat: Cat) {
        outputCat(cat: $cat) {
            name
        }
    }

    query takesDogBang($dog: Dog!) {
        outputDogBang(dog: $dog) {
            name
        }
    }

    query takesListOfPet($pets: [Pet]) {
        outputListOfPets(pets: $pets) {
            name
        }
    }

    query takesCatOrDog($catOrDog: CatOrDog) {
        outputCatOrDog(catOrDog: $catOrDog) {
            ... on Cat {
                name
            }
        }
    }

    fragment example on Bar {
        bar
    }

    subscription {
        ... example
        ... on Subscription {
            foo
        }
    }
    "#;

    let schema_ast = graphql_parser::parse_schema::<&str>(&schema).unwrap();
    let query_ast = graphql_parser::parse_query(&query).unwrap();

    c.bench_function("validate", |b| {
        b.iter(|| {
            black_box(litho_validation::validate(
                black_box(&schema_ast),
                black_box(&query_ast),
            ))
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
