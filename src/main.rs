use std::fmt::Formatter;
use std::fmt::Display;
use std::fmt::Debug;

fn main() {
    println!("{}", find_operations_for_value(vec![1, 2, 3], 6)
    .into_iter()
    .map(|x| {
        format!("{}", x)
    })
    .collect::<Vec<String>>()
    .join(", "));
}

#[derive(Clone, Debug)]
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

impl <T: Display> Display for Operation<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Operation::SingleOperand(value) => write!(f, "{}", value),
            Operation::Operation(operand1, operator, operand2) => write!(f, "{} {} {}", operand1, operator, operand2)
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
                    Operator::DIVIDE => if operand2 == 0 {
                        0
                    } else {
                        operand1 / operand2
                    },
                    Operator::MULTIPLY => operand1 * operand2
                }
            }
        }
    }
}

fn find_operations_for_value(operands: Vec<i32>, target_value: i32) -> Vec<Operation<i32>> {
    permutations(operands)
        .into_iter()
        .flat_map(|operands| generate_all_operations(operands))
        .filter(|x| x.evaluate() == target_value)
        .collect()
}

fn generate_all_operations<T: 'static + Clone + Debug>(
    operands: Vec<T>,
) -> impl Iterator<Item = Operation<T>> {
    return (1..=operands.len())
        .flat_map(move |size| generate_operations(operands[..size].to_vec()))
        .into_iter();
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

fn permutations<T: Clone>(vec: Vec<T>) -> Vec<Vec<T>> {
    if vec.len() == 1 {
        return vec![vec];
    }
    return (0..vec.len())
        .flat_map(|i| {
            let mut vec_without_i = vec.clone();
            let removed_element: T = vec_without_i.remove(i);
            let mut permutations_vec = permutations(vec_without_i);
            for permutation in &mut permutations_vec {
                permutation.push(removed_element.clone());
            }
            permutations_vec
        })
        .collect();
}
