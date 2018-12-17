extern crate clap;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;

fn main() {
    let matches = clap::App::new("chance")
        .version("1.0")
        .about("Solver for the chance game")
        .arg(
            clap::Arg::with_name("target")
                .short("t")
                .long("target")
                .help("Target value")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("values")
                .short("v")
                .long("values")
                .help("Values to use when searching for the target. Specified like --values=1,2,3")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("enable_associative_operation_filter")
                .long("enable_associative_operation_filter")
                .help("True to filter out similar operations (operations that use the same numbers and operators but in a different order)"),
        )
        .get_matches();
    let values: Vec<i32> = matches
        .value_of("values")
        .expect("Requires --values")
        .split(",")
        .map(|value| value.parse::<i32>().expect("Values must be integers"))
        .collect();
    let target: i32 = matches
        .value_of("target")
        .expect("Requires --target")
        .parse()
        .expect("Target must be an integer");
    let operations = find_operations_for_value(values, target);
    if matches.occurrences_of("enable_associative_operation_filter") > 0 {
        println!(
            "{}",
            format_operations(process_associative_operation_filter(operations))
        );
    } else {
        println!("{}", format_operations(operations));
    }
}

fn format_operations<T: Display>(operations: impl Iterator<Item = Operation<T>>) -> String {
    operations
        .into_iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join("\n")
}

fn process_associative_operation_filter<T: Hash + Eq + Clone>(
    operations: impl Iterator<Item = Operation<T>>,
) -> impl Iterator<Item = Operation<T>> {
    operations
        .map(|operation| (SimilarOperationKey::from(operation.clone()), operation))
        .collect::<HashMap<SimilarOperationKey<T>, Operation<T>>>()
        .into_iter()
        .map(|(_key, operation)| operation)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Operator {
    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Operator::ADD => write!(f, "+"),
            Operator::SUBTRACT => write!(f, "-"),
            Operator::MULTIPLY => write!(f, "*"),
            Operator::DIVIDE => write!(f, "/"),
        }
    }
}

impl Operator {
    fn values() -> Vec<Operator> {
        return vec![
            Operator::ADD,
            Operator::SUBTRACT,
            Operator::DIVIDE,
            Operator::MULTIPLY,
        ];
    }
}

#[derive(Clone)]
enum Operation<T> {
    SingleOperand(T),
    Operation(T, Operator, Box<Operation<T>>),
}

impl<T: Display> Display for Operation<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Operation::SingleOperand(value) => write!(f, "{}", value),
            Operation::Operation(operand1, operator, operand2) => {
                write!(f, "{} {} {}", operand1, operator, operand2)
            }
        }
    }
}

impl Operation<i32> {
    fn evaluate(&self) -> i32 {
        match self {
            Operation::SingleOperand(value) => value.clone(),
            Operation::Operation(operand1, operator, operand2) => {
                let operand2 = operand2.evaluate();
                match operator {
                    Operator::ADD => operand1 + operand2,
                    Operator::SUBTRACT => operand1 - operand2,
                    Operator::DIVIDE => {
                        if operand2 == 0 {
                            0
                        } else {
                            operand1 / operand2
                        }
                    }
                    Operator::MULTIPLY => operand1 * operand2,
                }
            }
        }
    }
}

fn find_operations_for_value(
    operands: Vec<i32>,
    target_value: i32,
) -> impl Iterator<Item = Operation<i32>> {
    power_set(operands)
        .flat_map(|sets| permutations(sets))
        .into_iter()
        .flat_map(|operands| generate_operations(operands))
        .filter(move |x| x.evaluate() == target_value)
}

fn generate_operations<T: 'static + Clone + Debug>(
    mut operands: Vec<T>,
) -> Box<Iterator<Item = Operation<T>>> {
    let first_operand = operands.remove(0);
    // Add one because we just removed a value.
    if operands.len() + 1 == 1 {
        Box::new(vec![Operation::SingleOperand(first_operand)].into_iter())
    } else {
        let sub_operations: Box<Iterator<Item = Operation<T>>> = generate_operations(operands);
        Box::new(sub_operations.flat_map(move |sub_operation| {
            let first_operand = first_operand.clone();
            Operator::values().into_iter().map(move |operator| {
                Operation::Operation(
                    first_operand.clone(),
                    operator.clone(),
                    Box::new(sub_operation.clone()),
                )
            })
        }))
    }
}

fn power_set<T: 'static + Clone>(vec: Vec<T>) -> impl Iterator<Item = Vec<T>> {
    if vec.len() >= 32 {
        panic!("Set is too large to generate power sets for.");
    }
    let base: i32 = 2;
    (0..(base.pow(vec.len() as u32))).map(move |bit_vector: i32| {
        vec.clone()
            .into_iter()
            .enumerate()
            .filter(|(index, _value)| (1 << index) & bit_vector != 0)
            .map(|(_index, value)| value)
            .collect::<Vec<T>>()
    })
}

fn permutations<T: 'static + Clone + Debug>(vec: Vec<T>) -> Box<Iterator<Item = Vec<T>>> {
    if vec.len() == 1 {
        Box::new(vec![vec].into_iter())
    } else {
        Box::new((0..vec.len()).flat_map(move |i| {
            let mut vec_without_i = vec.clone();
            let removed_element: T = vec_without_i.remove(i);
            permutations(vec_without_i).map(move |mut permutation| {
                permutation.push(removed_element.clone());
                permutation
            })
        }))
    }
}

// A key for a hash map that helps to deduplicate "similar" operations.
// Operations are considered similar if they contain the exact same operands and operators but in different orders.
#[derive(PartialEq, Eq)]
struct SimilarOperationKey<T: Hash + Eq> {
    operators: HashSet<Operator>,
    operands: HashSet<T>,
}

impl<T: Hash + Eq> Hash for SimilarOperationKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for operator in self.operators.iter() {
            operator.hash(state);
        }
        for operand in self.operands.iter() {
            operand.hash(state);
        }
    }
}

impl<T: Hash + Eq> From<Operation<T>> for SimilarOperationKey<T> {
    fn from(operation: Operation<T>) -> Self {
        match operation {
            Operation::SingleOperand(value) => SimilarOperationKey {
                operators: HashSet::new(),
                operands: vec![value].into_iter().collect(),
            },
            Operation::Operation(operand1, operator, operand2) => {
                let mut key: SimilarOperationKey<T> = (*operand2).into();
                key.operators.insert(operator);
                key.operands.insert(operand1);
                key
            }
        }
    }
}
