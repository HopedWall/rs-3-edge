use std::collections::VecDeque;

use crate::graph::Graph;
use crate::state::State;

#[derive(Debug)]
enum Inst {
    Recur(usize, usize),
    Loop(usize, usize, usize),
    Return(usize, usize),
}

type InstStack = VecDeque<Inst>;

fn run_inst(
    inst: Inst,
    stack: &mut InstStack,
    state: &mut State,
    graph: &Graph,
) {
    match inst {
        Inst::Recur(w, v) => {
            state.visited.insert(w);
            state.next_sigma[w] = w;
            state.next_on_path[w] = w;
            state.pre[w] = state.count;
            state.lowpt[w] = state.count;
            state.num_descendants[w] = 1;
            state.count += 1;

            graph[&w]
                .iter()
                .rev()
                .for_each(|edge| stack.push_front(Inst::Loop(w, v, *edge)));
        }
        Inst::Loop(w, v, u) => {
            state.degrees[w] += 1;

            if !state.visited.contains(&u) {
                stack.push_front(Inst::Return(w, u));
                stack.push_front(Inst::Recur(u, w));
            } else {
                // (w, u) outgoing back-edge of w, i.e. dfs(w) > dfs(u)
                if u != v && state.is_back_edge(w, u) {
                    if state.pre[u] < state.lowpt[w] {
                        state.absorb_path(w, state.next_on_path[w], None);
                        state.next_on_path[w] = w; // P_w in paper
                        state.lowpt[w] = state.pre[u];
                    }
                // (w, u) incoming back-edge of w, i.e. dfs(u) > dfs(w)
                } else if u != v {
                    state.degrees[w] -= 2;

                    if !state.is_null_path(w) {
                        let mut parent = w;
                        let mut child = state.next_on_path[w];

                        while !state.is_null_path(parent)
                            && state.pre[child] <= state.pre[u]
                        // child must have been visited before u
                            && state.pre[u] < state.pre[child] + state.num_descendants[child]
                        // child is still an ancestor of u
                        {
                            parent = child;
                            child = state.next_on_path[child];
                        }

                        // P_w[w..u] in paper
                        state.absorb_path(
                            w,
                            state.next_on_path[w],
                            Some(parent),
                        );

                        state.next_on_path[w] = if state.is_null_path(parent) {
                            w
                        } else {
                            state.next_on_path[parent]
                        }
                    }
                }
            }
        }
        Inst::Return(w, u) => {
            state.num_descendants[w] += state.num_descendants[u];

            if state.degrees[u] <= 2 {
                state.degrees[w] += state.degrees[u] - 2;
                state.add_component(u);

                state.path_u = if state.is_null_path(u) {
                    w // P_u = w + P_u
                } else {
                    state.next_on_path[u] // P_u
                };
            } else {
                // since degree[u] != 2, u can be absorbed
                state.path_u = u;
            }

            if state.lowpt[w] <= state.lowpt[u] {
                // w + P_u in paper
                state.absorb_path(w, state.path_u, None);
            } else {
                state.lowpt[w] = state.lowpt[u];
                // P_w in paper
                state.absorb_path(w, state.next_on_path[w], None);
                state.next_on_path[w] = state.path_u;
            }
        }
    }
}

pub fn three_edge_connect(graph: &Graph, state: &mut State) {
    let mut stack: InstStack = VecDeque::new();

    let nodes: Vec<_> = graph.keys().collect();
    for &n in nodes {
        if !state.visited.contains(&n) {
            stack.push_front(Inst::Recur(n, 0));
            while let Some(inst) = stack.pop_front() {
                run_inst(inst, &mut stack, state, graph);
            }
            state.add_component(n);
        }
    }
}
