# LIDO Provider chain proposal and votes processing (POC)


There is at least one problem with the current implementation of the proposals queue (see the `sudo_proposals_query` function). It does not support the deletion of proposals in the case of insufficient deposits. Given that this is a POC (Proof of Concept) contract, it was decided to leave the queue processing as is.

However, in the production version of the contract, it will be required to process the proposals queue more carefully and take into account the potential large number of underdeposited proposals.

One of the possible solutions is to make the window for querying proposals dynamic, so it keeps increasing until it reaches an empty proposal (if we are querying non-existent proposals via the query relayer, it returns an empty proposal with null-valued fields). After that, we need to monitor the current list of proposals for changes (if a proposal becomes null, it means that this proposal was removed due to insufficient deposit, and we can remove this proposal from the queue). This implies that we need to store the previous queue content to compare it with the current results.

Also, we need to define minimum and maximum window lengths, as well as the ability to manually control different aspects of the contract, to fix possible problems during an attack involving a large number of proposal creations.