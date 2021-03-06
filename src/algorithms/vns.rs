//! _variable neighborhood search_
use crate::{
    selectors::OperatorSelector, termination::TerminationCriteria, Evaluate, ImprovingHeuristic,
};

/// Implementation of _variable neighborhood search_ according to [here](https://en.wikipedia.org/wiki/Variable_neighborhood_search)
pub struct VariableNeighborhoodSearch<Solution, Selector: OperatorSelector<Solution>> {
    selector: Selector,
    terminator: Box<dyn TerminationCriteria<Solution>>,
}

/// Builder pattern to construct a _variable neighborhood search_ heuristic
pub struct VNSBuilder<Solution, Selector> {
    selector: Option<Selector>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    rng: Option<Box<dyn rand::RngCore>>,
}

impl<'a, Solution, Selector: OperatorSelector<Solution>> VNSBuilder<Solution, Selector> {
    /// Set operator selector
    pub fn selector(mut self, selector: Selector) -> Self {
        self.selector = Some(selector);
        self
    }

    /// Set termination criteria
    pub fn terminator<T: TerminationCriteria<Solution> + 'static>(mut self, terminator: T) -> Self {
        self.terminator = Some(Box::new(terminator));
        self
    }

    /// Set source of randomness
    pub fn rng<T: rand::RngCore + 'static>(mut self, rng: T) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }

    /// Construct the specified heuristic.
    pub fn build(self) -> VariableNeighborhoodSearch<Solution, Selector> {
        VariableNeighborhoodSearch {
            selector: self.selector.expect("Did not specify an operator selector"),
            terminator: self
                .terminator
                .expect("Did not specify termination criteria"),
        }
    }
}

impl<'a, Solution, Selector: OperatorSelector<Solution>>
    VariableNeighborhoodSearch<Solution, Selector>
{
    /// Return a builder to simplify the specification.
    pub fn builder() -> VNSBuilder<Solution, Selector> {
        VNSBuilder {
            selector: None,
            rng: None,
            terminator: None,
        }
    }
}

impl<Solution, Selector> ImprovingHeuristic<Solution>
    for VariableNeighborhoodSearch<Solution, Selector>
where
    Selector: OperatorSelector<Solution>,
{
    /// Accept iff candidate is better than the incumbent.
    fn accept_candidate(&self, candidate: &Solution, incumbent: &Solution) -> bool
    where
        Solution: Evaluate,
    {
        candidate.evaluate() < incumbent.evaluate()
    }

    /// Test whether the termination criteria are fulfilled.
    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(incumbent)
    }

    /// Select operator and get the best neighbor if ```solution```.
    fn propose_candidate(&self, solution: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let operator = self.selector.select(&solution);
        operator.find_best_neighbor(solution)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        algorithms::vns::VariableNeighborhoodSearch, selectors::SequentialSelector,
        termination::IterationTerminator, test::*, ImprovingHeuristic,
    };

    #[test]
    fn vns_single_operator1() {
        let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
        let iterations_max = 10;

        let operator = NeighborsUpUntilN::new(&numbers, 1);
        let vns = VariableNeighborhoodSearch::builder()
            .selector(SequentialSelector::new().option(operator))
            .terminator(IterationTerminator::new(iterations_max))
            .build();

        let initial_solution = Number::new(0, numbers[0]);
        let vns_solution = vns.optimize(initial_solution);

        assert_eq!(vns_solution.index(), 2)
    }

    #[test]
    fn vns_single_operator2() {
        let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
        let iterations_max = 10;

        let operator = NeighborsUpUntilN::new(&numbers, 3);
        let vns = VariableNeighborhoodSearch::builder()
            .selector(SequentialSelector::new().option(operator))
            .terminator(IterationTerminator::new(iterations_max))
            .build();

        let initial_solution = Number::new(0, numbers[0]);
        let vns_solution = vns.optimize(initial_solution);

        assert_eq!(vns_solution.index(), 6)
    }

    #[test]
    fn vns_multiple_operators1() {
        let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
        let iterations_max = 10;

        let operator1 = NeighborsUpUntilN::new(&numbers, 1);
        let operator2 = NeighborsUpUntilN::new(&numbers, 3);
        let vns = VariableNeighborhoodSearch::builder()
            .selector(
                SequentialSelector::new()
                    .option(operator1)
                    .option(operator2),
            )
            .terminator(IterationTerminator::new(iterations_max))
            .build();

        let initial_solution = Number::new(0, numbers[0]);
        let vns_solution = vns.optimize(initial_solution);

        assert_eq!(vns_solution.index(), 2)
    }

    #[test]
    fn vns_multiple_operators2() {
        let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
        let iterations_max = 10;

        let operator1 = NeighborsUpUntilN::new(&numbers, 1);
        let operator2 = NeighborsUpUntilN::new(&numbers, 4);
        let vns = VariableNeighborhoodSearch::builder()
            .selector(
                SequentialSelector::new()
                    .option(operator1)
                    .option(operator2),
            )
            .terminator(IterationTerminator::new(iterations_max))
            .build();

        let initial_solution = Number::new(0, numbers[0]);
        let vns_solution = vns.optimize(initial_solution);

        assert_eq!(vns_solution.index(), 7)
    }
}
