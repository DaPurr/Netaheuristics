## VNS

1. incumbent = initial solution

2. loop
   
   1. select candidate
   
   2. if candidate better than incumbent
      
      1. incumbent = candidate
      
      2. reset neighborhood
   
   3. else
      
      1. if last neighborhood return incumbent
      
      2. select next neighborhood

## SA

1. incumbent = initial solution

2. loop
   
   1. select candidate
   
   2. if candidate is accepted wrt to acceptance probability
      
      1. incumbent = candidate

## Tabu search

1. incumbent = initial solution

2. loop
   
   1. select candidate which is not tabu, or is aspirated
   
   2. if candidate is better than incumbent
      
      1. incumbent = candidate
      
      2. update tabu list

## LNS

1. incumbent = initial solution

2. best solution = incumbent

3. loop
   
   1. select candidate by destroying and then repairing the incumbent
   
   2. if candidate is accepted
      
      1. incumbent = candidate
   
   3. if candidate is better than best solution
      
      1. best solution = candidate

## General idea for framework

1. incumbent = initial solution

2. best solution = initial solution

3. loop while termination criteria not satisfied
   
   1. select candidate
   
   2. if accept candidate
      
      1. incumbent = candidate
   
   3. if candidate is better than best solution
      
      1. best solution = candidate

4. return best solution
