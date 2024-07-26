# Drop protocol design

This Quint specification models the main functionality of Drop's protocol. Apart from a few libraries ([basicSpells.qnt](quint-model/basicSpells.qnt), [extraSpells.qnt](quint-model/extraSpells.qnt), [dec.qnt](quint-model/dec.qnt)), it includes the following files:

- [drop.qnt](quint-model/drop.qnt): defines model in terms of functional transformations (eg, how a state is transformed upon a tick action). 
- [dropEvolution.qnt](quint-model/dropEvolution.qnt): defines how the protocol may evolve - what is the initial state and what are possible transitions.
- [runs.qnt](quint-model/runs.qnt): includes sample sequences of actions to run in order to interact with the model
- [types.qnt](quint-model/types.qnt): includes all data types definitions
- [simulationValues.qnt](quint-model/simulationValues.qnt): includes simulation parameters as well as the initial state
- [constants.qnt](quint-model/constants.qnt): constants user in the model
- [helpers.qnt](quint-model/helpers.qnt): helper functions

## Scope

The specification models the design of a liquid staking protocol. It models main protocol actions: *ticks*, *advancing time*, *receiving assets from pump* and *transfering funds to staker ICA* (we call it *delegator ICA* as per the glossary section in the docs). Also, it models user actions, such as *staking*, *unstaking* and *withdrawing*, as well as actions that revolve around *interchain actions* and *remote queries*. The following diagram represents the finite state machine design.

![FSM diagram](diagram/diagram.png "FSM diagram")

## Model limitations and differences

The current model makes several assumptions:

- Rewards are modeled as constant amount (they do not depend on the amount bonded)
- This model represents the FSM design without:
  - LSM shares features
  - Non native token rewards
- We do not model different validators' behavior. Thus, slashing is done in a simple way: each time a `slashing` action happens, all not-yet-unbonded funds are slashed by a constant ratio. 

Also, the current model has several differences compared to the implementation:

- Some non-core-contract operations happen atomically, such as *transfering funds to staker ICA*
- We keep more than one ICA request and response in their respective queues. Idea behind this approach is to be able to find executions in which there is accumulated more than one ICA response (which should never happen).
- We check the type of response when receiving an ICA response. This is not the case in the protocol implementation. The reason is again to detect anomalies early in the model evolution.

## Definitions

Tick is permissionless. Ticks cause state transitions in finite state machine. States are represented with circles in the diagram. FSM states in our model are: `IDLE`, `WAITING_CLAIM_WITHDRAW`, `WAITING_DELEGATE_USER_FUNDS`, `WAITING_DELEGATE_REWARDS` and `WAITING_UNDELEGATE`. They are saved in the `fsmState` property of the `dropState`.
System state definition is placed in [types.qnt](quint-model/types.qnt) file and consists of two substates: `dropState` and `envState`.

User actions are not presented on the diagram, and they are state independent.

All possible actions in our model can be seen in the action `step` of the [dropEvolution.qnt](quint-model/dropEvolution.qnt) file. Those actions in turn call state transformations as defined in [drop.qnt](quint-model/drop.qnt).

## How To Run It

### Requirements

- [Quint](https://github.com/informalsystems/quint) (tested with v0.20.0)

### Resources

- [Quint cheatsheet](https://github.com/informalsystems/quint/blob/main/doc/quint-cheatsheet.pdf)
- [Quint tutorials](https://github.com/informalsystems/quint/tree/main/tutorials)

### Interact with the model in a REPL

Switch directory to `quint-model`.

```shell
# Load the main file
quint --r dropEvolution.qnt::dropEvolution
```

This command will open Quint REPL. Inside the Quint REPL:

Initialize the state by executing the `init` action
```quint
init
```
You may receive a warning about uninitialized actions. 
These are used as book-keeping variables for simulations so ignore the warning for the moment.

Run a step which executes one of the possible actions. Repeat this proces several times.
```quint
step
```
Evaluate the drop state (or any other variable)
```quint
state
state.dropState.totalLdInCirculation
```

### Running sample runs

[runs.qnt](quint-model/runs.qnt) supplies several test runs. Enter Quint REPL for [runs.qnt](quint-model/runs.qnt) file.

```shell
quint -r runs.qnt::runs
```

Choose one of them to run in Quint REPL.

```quint
user_withdraw_run
```

Alternatively, you can choose to run all sampled runs with this command that matches on names of the runs that contain *run*:

```shell
quint test --match=run --max-samples=1 runs.qnt
```
Here, `--max-samples` defines a number of times for each run to be run. 
The current existing runs are deterministic, so using value `1` makes sense.
(For nondeterministic runs, go with the default value of `10000`.)

After executing run, you can freely execute any desired action in order to expand scenario.
That way, you can interact with the model by using runs to reach an interesting initial state and working from there.

### Run the Simulator and Check Invariants

With the following command, the simulator will explore 1000 evolutions of 50 steps. 
At every step, it will check whether the property `[invariant_name]` holds (and report if it does not).
In [dropEvolution.qnt](quint-model/dropEvolution.qnt), we provide a couple of invariant examples, such as `no_two_ica_messages`.

```shell
quint run --max-samples=1000 --max-steps=50 --invariant=[invariant_name] dropSimulation.qnt
```

You may notice we are using the `dropSimulation.qnt` file. This file is a wrapper around `dropEvolution.qnt`, which only provides two more book-keeping variables, to let us inspect all the actions taken.

If no invariant is supplied, the simulator will simply explore possible model evolutions.

In case you want to see what steps have been taken, we suggest you use ITF (*Informal trace format*) traces. Traces store every step that has been executed in the run. In order to to visualize trace, you should also install the [VSCode plugin for visualizing traces](https://marketplace.visualstudio.com/items?itemName=informal.itf-trace-viewer).

The following command saves all steps of one of the executed runs into ITF trace `out.itf.json`:

```shell
quint run --max-samples=1000 --max-steps=50  --out-itf=out.itf.json --mbt dropSimulation.qnt
```

Also it is possible to swap steps and use a custom step. To do so, use parameter `--step` and set custom step as `[custom_step]`.

```shell
quint run --max-samples=1000 --max-steps=50  --out-itf=out.itf.json --mbt --step=[custom_step] dropSimulation.qnt
```
