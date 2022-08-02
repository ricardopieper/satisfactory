use core::num;
use std::{
    collections::{BTreeMap, HashSet},
    panic,
};

use rand::{prelude::SmallRng, seq::SliceRandom, Rng, SeedableRng};

#[derive(Eq, PartialEq)]
struct User {
    id: u32,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.id.to_string())
    }
}

#[derive(Debug)]
enum Constraint {
    DisallowTogether(u32, u32),
}

#[derive(Debug)]
struct Satisfactory<'user> {
    data: Vec<&'user User>,
    group_size: u32,
    constraints: Vec<Constraint>,
}

impl<'a> Satisfactory<'a> {
    fn new(data: Vec<&'a User>) -> Self {
        Self {
            data,
            group_size: 0,
            constraints: vec![],
        }
    }

    fn groupings_of(&mut self, size: u32) -> &mut Self {
        self.group_size = size;
        self
    }

    fn add_constraint(&mut self, constraint: Constraint) -> &mut Self {
        self.constraints.push(constraint);
        self
    }
}

struct SatisfactoryRunner<'users> {
    data: Satisfactory<'users>,
    constraints_map: BTreeMap<u32, HashSet<u32>>,
    calls: std::cell::Cell<u32>,
    debug: bool,
}

impl<'users> SatisfactoryRunner<'users> {
    fn new(input: Satisfactory<'users>) -> SatisfactoryRunner<'users> {
        SatisfactoryRunner {
            data: input,
            constraints_map: BTreeMap::new(),
            calls: std::cell::Cell::new(0),
            debug: true,
        }
    }

    fn build_contraints_map(&mut self) {
        for user in self.data.data.iter() {
            self.constraints_map
                .insert(user.id.clone(), HashSet::<u32>::new());
        }

        for constraint in self.data.constraints.iter() {
            match constraint {
                Constraint::DisallowTogether(a, b) => {
                    let set = self
                        .constraints_map
                        .get_mut(a)
                        .expect("ID in constraint not found in data");
                    set.insert(b.clone());

                    let set = self
                        .constraints_map
                        .get_mut(b)
                        .expect("ID in constraint not found in data");
                    set.insert(a.clone());
                }
            }
        }
    }

    /*fn run<Rand: Rng>(&mut self, rng: &mut Rand) -> Vec<Vec<&String>> {
    if self.data.data.len() % 2 != 0 {
        panic!("Cannot group odd lmao")
    }
    self.calls.set(0);
    self.data.data.shuffle(rng);
    self.build_contraints_map();
    let mut solution = vec![];
    let data = &self.data.data[..];

    for user in data.iter() {
        let rest = data.iter().filter(|x| x.id != user.id).collect::<Vec<_>>();
        let rest_slice = &rest[..];
        self.backtrack(rng, user, rest_slice, &mut solution, 0);
    }

    return solution;
    }*/

    fn run<Rand: Rng>(&mut self, rng: &mut Rand) -> Vec<Vec<&u32>> {
        if self.data.data.len() % 2 != 0 {
            panic!("Cannot group odd lmao")
        }
        self.calls.set(0);
        self.build_contraints_map();
        //println!("{:?}", self.constraints_map);

        self.data.data.shuffle(rng);
        let mut solution = vec![];
        let unsolved = &self.data.data[..];
        if self.debug {
            println!("Unsolved: {unsolved:?}");
        }
        //root iteration
        for user in &self.data.data[..] {
            let others = unsolved
                .iter()
                .filter(|x| x.id != user.id)
                .map(|x| *x)
                .collect::<Vec<_>>();

            if self.recurse(&mut solution, *user, &others, 0, rng) {
                return solution;
            }
        }
        panic!("Unsolvable?");
    }

    fn recurse<'s, 'iteration, Rand: Rng>(
        &'s self,
        solution: &'s mut Vec<Vec<&'users u32>>,
        me: &'users User,
        rest: &'iteration [&'users User],
        depth: u32,rng: &mut Rand) -> bool {

        //while !self.solution_is_plausible(&rest, solution) {
        //    solution.pop();
        //    return false;
        //}

        self.calls.set(self.calls.get() + 1);

        let indent = " ".repeat(depth.min(30) as usize);
        //if self.calls.get() % 10 == 0 {
        let iter = self.calls.get();
        if iter == 30493 {
            println!("Debug!");
        }
        if self.debug {
            println!("Current Solution: {solution:?}, user = {me:?}, rest = {rest:?} iteration {iter}", iter = self.calls.get());
        }
        //}
        let forbidden_for_me = self.constraints_map.get(&me.id).expect("no way!");

        //base case: just check if the 2 are compatible and do a quick return if they aren't.
        //If they are compatible, we're in luck. Add to the solution and return true. Avoid all the dancing
        //that comes with groupables, etc etc
        if rest.len() == 1 {
            if forbidden_for_me.contains(&rest[0].id) { 
                return false 
            } else {
                solution.push(vec![&me.id, &rest[0].id]);
                return true;
            }
        }


        let mut groupables: Vec<_> = rest
            .iter()
            .filter(|user| {
                if user.id == me.id {
                    return false;
                }

                if forbidden_for_me.contains(&user.id) {
                    return false;
                }

                //iterate over solution to find whether this user is already grouped (on the current potential solution)
                for group in solution.iter() {
                    for group_user in group.iter() {
                        if user.id == **group_user {
                            return false;
                        }
                    }
                }
                //good to go, we can pair with this user
                return true;
            })
            .map(|x| *x)
            .collect();

        if self.debug {
            println!(
                "{indent}Groupables for user {user}: {groupables:?}",
                user = me.id
            );
        }
        
        if groupables.len() == 0 {
            if self.debug {
                println!("{indent}No groupables for user {user}", user = me.id);
            }

            //if no more groupables, check solution size
            //if solved return true
            let users_solved_count: usize = solution.iter().map(|x| x.len()).sum();
            if users_solved_count == self.data.data.len() {
                return true;
            } else {
                //no point in continuing, solution for me is impossible.
                if self.debug {
                    println!("{indent}No point in continuing, constraints disallow this solution for user {me:?}, returning false");
                }
                //solution.pop();
                return false;
            }
        }

        //attempt using groupables in solution
        if self.debug {
            println!(
                "{indent}Trying solutions with user {me:?} using the groupables {groupables:?}"
            );
        }

        for groupable in groupables {
            solution.push(vec![&me.id, &groupable.id]);
            if self.debug {
                println!("{indent}Try: solution: {solution:?}");
            }
            let mut unsolved = self
                .data
                .data
                .iter()
                .filter(|x| {
                    for group in solution.iter() {
                        for user in group.iter() {
                            if **user == x.id {
                                return false;
                            }
                        }
                    }
                    return true;
                })
                .map(|x| *x)
                .collect::<Vec<_>>();

            if self.debug {
                println!("{indent}Unsolved: {unsolved:?}");
            }

            if unsolved.len() == 0 {
                return true;
            }

            //Among the unsolvables, if there is even a single element that is not compatible with *any* of the remaining elements, 
            //we are doomed to fail. We will not be able to continue.
            //Therefore, we first need to check if a solution is even marginally possible, i.e. every remaining element can be matched
            //with at least one other element
            //for unsolved_user in unsolved.iter() {
            
            let mut bad = false;
            while !self.solution_is_plausible(&unsolved, solution) {
                solution.pop();
                bad = true;
            }
            if bad { return false };

            //if there are only 2 remaining unsolvables, what will end up happening is 
            //that it will try 
            for unsolved_user in unsolved.iter() {
                if self.debug {
                    println!(
                        "{indent}Trying unsolved user: {user:?}",
                        user = unsolved_user.id
                    );
                }

                let others = unsolved
                    .iter()
                    .filter(|x| x.id != unsolved_user.id)
                    .map(|x| *x)
                    .collect::<Vec<_>>();

                if self.recurse(solution, *unsolved_user, &others, depth + 1, rng) {
                    return true;
                }
            }

            if self.debug { println!("{indent}No solution found, popping solution") }
            solution.pop(); //no luck, try again
        }

        //try again with the other users
        for user in rest.iter() {
            if self.debug {
                println!(
                    "{indent}Trying solving the rest, user {user:?}",
                    user = user.id
                );
            }

            let mut others = rest
                .iter()
                .filter(|x| x.id != user.id)
                .map(|x| *x)
                .collect::<Vec<_>>();

            if self.recurse(solution, *user, &others, depth + 1, rng) {
                return true;
            }
        }

        return false;
    }


    fn solution_is_plausible(&self, unsolved: &[&User], candidate_solution: &mut Vec<Vec<&u32>>) -> bool {
        if candidate_solution.is_empty() {return true; }
        for unsolved_user in unsolved.iter() {
            let forbidden_for_unsolved_user = self.constraints_map.get(&unsolved_user.id).expect("no way!");

            //replace with take/skip/take maybe based on index?
            let has_groupable = unsolved
                .iter()
                .filter(|x| x.id != unsolved_user.id)
                .any(|u| {
                    if forbidden_for_unsolved_user.contains(&u.id) {
                        return false;
                    }
    
                    //iterate over solution to find whether this user is already grouped (on the current potential solution)
                    for group in candidate_solution.iter() {
                        for group_user in group.iter() {
                            if u.id == **group_user {
                                return false;
                            }
                        }
                    }
                    //good to go, we can pair with this user
                    return true;
                });
            
            if !has_groupable {
                //I tried so hard and got so far, in the end this solution doesn't matter
                return false;
            }
        }
        return true;
    }

    /*
    fn backtrack<'a, Rand>(
        &'a self,
        rng: &mut Rand,
        me: &'a User,
        others: &[&'a User],
        solution: &mut Vec<Vec<&'a String>>,
        depth: u32,
    ) -> bool
    where
        Rand: Rng,
    {
        /*if self.calls.get() > 10000000 {
        //println!("Time limit");
        return false;
        }*/
        self.calls.set(self.calls.get() + 1);

        let users_solved_count: usize = solution.iter().map(|x| x.len()).sum();

        if users_solved_count == self.data.data.len() {
            return true;
        }

        //find a user that has not been solved yet
        let users_not_solved = self
            .data
            .data
            .iter()
            .filter(|x| {
                for group in solution.iter() {
                    for user in group.iter() {
                        if **user == x.id {
                            return false;
                        }
                    }
                }
                return true;
            })
            .collect::<Vec<_>>();

        if users_not_solved.len() == 0 {
            panic!("could not find unsolved user, this should be impossible");
        }

        let forbidden_for_me = self.constraints_map.get(&me.id).expect("no way!");

        let mut groupables: Vec<_> = self
            .data
            .data
            .iter()
            .filter(|user| {
                if user.id == me.id {
                    return false;
                }

                if forbidden_for_me.contains(&user.id) {
                    return false;
                }

                //iterate over solution to find whether this user is already grouped (on the current potential solution)
                for group in solution.iter() {
                    for group_user in group.iter() {
                        if user.id == **group_user {
                            return false;
                        }
                    }
                }
                //good to go, we can pair with this user
                return true;
            })
            .collect();
        groupables.shuffle(rng);
        /*println!(
        "groupables for {me}: {groupables}",
        me = me.id,
        groupables = groupables.len()
        );*/

        let mut current_permutation = vec![&me.id];

        return self.permutations(
            rng,
            solution,
            &groupables,
            &mut current_permutation,
            0,
            self.data.group_size as usize - 1 as usize,
            depth,
        );
    }

    fn permutations<'a, Rand: Rng>(
        &'a self,
        rng: &mut Rand,
        solution: &mut Vec<Vec<&'a String>>,
        groupables: &[&'a User],
        current_permutation: &mut Vec<&'a String>,
        left: usize,
        permutation_size: usize,
        depth: u32,
    ) -> bool {
        if permutation_size == 0 {
            solution.push(current_permutation.clone());

            let users_solved_count: usize = solution.iter().map(|x| x.len()).sum();
            if users_solved_count == self.data.data.len() {
                return true;
            }

            solution.pop();

            for user in groupables.iter() {
                let rest = groupables
                    .clone()
                    .into_iter()
                    .filter(|x| x.id != user.id)
                    .map(|x| *x)
                    .collect::<Vec<_>>();
                let rest_slice = &rest[..];
                if self.backtrack(rng, user, rest_slice, solution, 0) {
                    return true;
                }
            }
        }

        for groupable_user in groupables.iter().skip(left) {
            current_permutation.push(&groupable_user.id);
            if self.permutations(
                rng,
                solution,
                groupables,
                current_permutation,
                left + 1,
                permutation_size - 1,
                depth,
            ) {
                return true;
            } else {
                current_permutation.pop();
            }
        }
        return false;
    }
     */
}

/*
fn main() {
    let mut rng = SmallRng::seed_from_u64(0xc18131e85914);
    let mut users = vec![];
    for i in 1..=6 {
        users.push(User { id: i })
    }
    let mut satisfactory = Satisfactory::new(users.iter().map(|x| x).collect());
    satisfactory.groupings_of(2);

    for i in 1..=6 {
        satisfactory
            .add_constraint(Constraint::DisallowTogether(i, i));
    }

    let mut runner = SatisfactoryRunner::new(satisfactory);
    let result = runner.run(&mut rng);


    if result.len() == 0 {
        panic!("No solution");
    } else {
        println!("Solved: {result:?}")
    }
}
*/

fn main() {
    let runs = 1;
    let num_users = 20;
    let rooms = num_users;
    let mut failures = vec![];
    for r in 0..runs {
        println!("Running {r}...");

        for i in (num_users..=num_users).step_by(2) {
            let mut rng = SmallRng::seed_from_u64(0xc18131e85914);
            let mut constraints: Vec<Vec<u32>> = vec![];
            let max_sessions = (i - 1).min(rooms);
            for s in 1..=max_sessions {
                println!("Running {i} users in session {s} for run {r}");

                let mut users = vec![];
                for i in 1..=i {
                    users.push(User { id: i })
                }

                let mut satisfactory = Satisfactory::new(users.iter().map(|x| x).collect());
                satisfactory.groupings_of(2);
            

                for c in constraints.iter() {
                    satisfactory
                        .add_constraint(Constraint::DisallowTogether(c[0].clone(), c[1].clone()));
                }
                for i in 1..=i {
                    satisfactory.add_constraint(Constraint::DisallowTogether(i, i));
                }

                let mut runner = SatisfactoryRunner::new(satisfactory);
                runner.debug = s == 0;
                
                
                let result = runner.run(&mut rng);

                if result.len() == 0 {
                    println!("No solution for {i} users in {s} sessions");
                    panic!("No solution for {i} users in {s} sessions");
                    failures.push((i, s));
                    continue;
                } else {
                    println!("SOLUTION FOUND: {result:?}");
                }

                let mut counter = BTreeMap::new();
                for r in result.iter() {
                    let mut vec = vec![];
                    for u in r {
                        counter
                            .entry(u.to_owned())
                            .and_modify(|c| *c += 1)
                            .or_insert(1);

                        vec.push(**u);
                    }
                    constraints.push(vec);
                }

                if let Some((user, num)) = counter.iter().find(|c| *c.1 > 1) {
                    panic!("{user} appears {num} times :(")
                }
                let calls = runner.calls.get();
                println!("Solved in {} iterations", calls);
            }
        }
    }

    println!("Failures: {failures:?}");

    let differences = failures
        .iter()
        .map(|(users, sessions)| users - sessions)
        .collect::<Vec<_>>();

    println!("Differences: {differences:?}");
}
