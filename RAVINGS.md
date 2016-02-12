Ravings about R4
================

This is a collection of my thoughts on 'design' (read: ravings) and
intentions for the R4 microkernel. Anything written here is probably insane
and is not intended to make sense.

Caps
Read only kernel caps
Read write proxy caps
All caps have TTL which must decrement in the chain
Can link to turn proxy into real cap, if all caps in chain have right.
Intermediate caps now no longer revoke
Proxy caps have right to do single use operations or modify source caps (if
frame caps can only be mapped once)
Proxy cap needs to be in page marked (avl bits) to say it has proxy caps so
you cannot guess for random memory to gain privilege
Two kinds of proxy caps. Ones that defer rights elsewhere, and ones that
just point to and consume another right. Latter does not need to be in a
page marked as special. Do you even need the latter?
Invoke implies your PD, can explicitly invoke another PD, but must be a
proxy cap
Split kernel objects into two parts. One that must in kernel object memory
and not shared by userspace, and one that is a reference to a portion of
memory that a user can have mapped in. The latter representing object state
the user can read/write and hence may colocate many on one physical frame.

Talk about other address spaces for proxy caps by asid (window index) then
cap index inside there. This avoids traversing heap to determine if pointer
is an object. IPC is always asynchronous, unless waiting for single badge
then can opt to receive IPC up to length n. N is very small and this is a
minimal chnall to share asids (session ids) and proxy cap locations.

IPC split into 'queued' and 'unqueued'. Unqueued == pure notification.
queued implies the caller if sending a unique message, and might also be
sending its scheduling context to fullfill the request. 

Clustering ETC

Use a 'clustered' biglock kernel. The kernels are NOT complete multikernels.
Can share resources (see later) and could use the same code etc (probably
want to replicate for memory locality anyway).
A 'cluster' represents a domain in which memory and hardware resources are
considered 'uniform enough' to present a unified environment to user level.
Clusters may need grouping into something larger if they clusters still share
some hardware (I don't know if machines exist that have separate sockets
that share the same TLB space with respect to HW ASIDs). Assuming that isn't
true then a coherence domain would typically be a single CPU package.
Capabilities are (for this discussion) local to a single cluster. Threads
and address spaces can be scheduled on any hardware thread in the cluster
they were created in, but not transfered anywhere else.
Intention of kernel design
* A cluster has a single lock (more or less) for handling all internal
  state. Its internal state is still replicated per core for efficiency
* Whilst each cluster *could* access the state of another, the state cannot
  be a single giant array as the memory backing each cluster should be local
  to that cluster.
* Software ASIDs (used for proxy cap invocation) are therefore not valid
  between different clusters.
* Capablities can be explicitly transfered to another core. A reference gets
  left locally pointing to that remote core. Only untyped memory can be sent
  to another core.
* Some kind of primitive to do signalling across cores. Probably implemented
  as IPI on sender and regular interrupt on the receiver


Remote cap operations. Single (or multi) slot buffer for communication with
other cores. If user does an operation that would enqueue another request
the user gets blocked waiting. i.e. the ability to do this represents a
DoS channel that should only be in the hands of trusted servers. The ability
to send capabilities (since they result in these DoS operations upon revoke)
is also a trusted (requires a special cap) operation.


Some kind of story for properly doing interrupt priorities? i.e. not fucking
it up / ignoring it like in seL4.
