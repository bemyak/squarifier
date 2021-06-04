# Token Squarifier
This is a tiny project that help me run my D&D and Pathfinder games at Roll20.

## Where?

You can try it at https://squarifier.ml

## What?

In short, it dynamically turns this:

![Qurashith](https://2e.aonprd.com/Images/Monsters/Qurashith.png)

into this:

![Squarified Qurashith](https://squarifier.ml/aHR0cHM6Ly8yZS5hb25wcmQuY29tL0ltYWdlcy9Nb25zdGVycy9RdXJhc2hpdGgucG5n.png)

The image got cropped and made into a square.

## Why?
There are few problems with Roll20 that I was constantly facing:
1. Too small storage (100 Mb for images)
1. When you insert non-rectangular token, it gets stretched in order to fill the
square

I found the solution for the first problem, which is described at the end of
my blog post [Rolling like a Pro](https://bemyak.ml/dev/rolling-like-a-pro/).
In short, I'm now able to insert images as links, which doesn't eat any space at
all.

But now the second problem got much more worse: you rarely can find a square
image on the internet, so you need to download it, crop, make square and this
nullifies any win from the #1, because you don't have an image URL now, and you
need to upload it again.

To solve this, this project was created. It acts like a proxy that changes your
images in the way you'd probably want it: cropped and square. 🎉

## How?
We need to process images really quickly in order to be able to respond as fast
as possible. We still need to:
1. Get the request
1. Download requested image, while the incoming request is waiting
1. Process it
1. Answer the request with a processed image

So, the overall latency is around `2 * original_request_latency`

To minimize it further you can run `Squarifier` locally or on some nearby
server that has public IP and/or DNS name. To see how the infrastructure is set
up, check my [infrastructure](https://gitlab.com/bemyak/infrastructure)
repository.
