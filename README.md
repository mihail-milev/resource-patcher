# Important

This is a WIP! I do not recommend using it in production environments. It is still being worked on (see TODOs below) and I shall not be held responsible for any damage done by this source code in your environment (see License below).

# Introduction

This is a small "daemon" for Kubernetes-based clusters. It searches for "ResourcePatch" CRD objects and if it finds any in the cluster, it parses them and tries to apply the patch to the target object. A target object can be any object in a Kubernetes cluster.

# Installation

1. Apply the CRD from the yaml/crd.yaml file
2. Apply the service account from the yaml/sa.yaml file
3. Apply the role binding from the yaml/rb.yaml file
4. Create a deployment, which runs the executable file. The deployment must use the service account created above, in order to have rights for patching. 

Now "ResourcePatch" objects may be created. A sample structure for one can be found in yaml/test.yaml

# Alternatives

I am aware only of the [Resource Locker Operator](https://github.com/redhat-cop/resource-locker-operator), which was very complicated for my needs, so I decided to write a simpler one, which can also be deployed on vanilla Kubernetes clusters without much hassle.

# Why Rust?

1. I like challenges
2. I wanted to work on my Rust skills
3. I got bored of Go
4. I wanted to see if Rust is possible in the Kubernetes world
5. I like Rust (and Go; don't get me wrong ;) )

# TODOs

1. TESTS: there is not a single test case here. That's crazy!
2. Dockerfile and Buildah scripts
3. Proper CRD definition
4. Refresh the list of resource types regularly. Currently it is done only when the program starts, which means, if a CRD is added after that, the program will not be able to patch it
5. Proper documentation

# License

Every piece of code in this repo is licensed under the MIT license. Obtain a copy of it from the LICENSE.md file
