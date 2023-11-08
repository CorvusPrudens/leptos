# Leptos on AWS Lambda with Axum

This example shows how to deploy a Leptos Axum SSR application to an [AWS Lambda function](https://aws.amazon.com/lambda/).
It builds on top of the [Leptos Axum Starter Template](https://github.com/leptos-rs/start-axum),
and many of the same considerations apply here.

## Running Your project

On your local machine, you can run the project with `cargo-leptos`:

```bash
cargo leptos watch
```

The `--release` flag enables the [axum-aws-lambda](https://github.com/lazear/axum-aws-lambda) layer,
which only works within the AWS Lambda environment,
so it should be avoided locally.

## Deploying Your Project

To build and deploy your project to AWS, you'll need [cargo-lambda](https://www.cargo-lambda.info/).
They provide [installation instructions](https://www.cargo-lambda.info/guide/installation.html) on their site.

Let's start by building the project with `cargo-leptos`:

```bash
cargo leptos build --release
```

We won't use the server binary that it builds, since
the Lambda function requires a particular architecture that
`cargo-lambda` will handle for us. If you'd rather not build the server twice,
you'll have to manage the wasm build and optimization yourself.

Next, let's build the production server binary:

```bash
LEPTOS_OUTPUT_NAME=aws-lambda cargo lambda build --no-default-features --features=ssr --release
```

This should produce a binary at `target/lambda/aws-lambda/bootstrap`.
`Cargo.toml` exposes all the required environment variables to `cargo-lambda`
so that the server can run in production.

Finally, we can deploy the project to AWS:

```bash
cargo lambda deploy --include target/site --enable-function-url
```

After a few seconds, `cargo-lambda` should print out the URL of your deployed function!

## Notes

### Credentials

You'll need AWS credentials with some permissions for IAM and Lambda operations.
`cargo-lambda` provides the [minimum requirements here](https://www.cargo-lambda.info/commands/deploy.html#user-profile).

Setting up permissions can be a bit onerous if this is your first time working with AWS.
For a quick and dirty setup, you can:
1. Create a new user in the IAM service (Access Management > Users)
2. Click "Attach policies directly" on the "Set permissions" page
3. Add the "AWSLambda_FullAccess" and "IAMFullAccess" policies, and complete the user creation
4. Create an access key for the user (don't worry about the warning)
5. Place the access key and secret key in `~/.aws/credentials` (or wherever the
  appropriate location is for your system):

```
[default]
aws_access_key_id = AKIAQYLPMN5HCTNK35FD
aws_secret_access_key = rbWHpaI/lJnXdLteWHNnTVZpQztMB2+pdbb+KVgr
````

### Optimizations

Serving static files from a lambda function is not the best approach.
Ideally, you should upload your files to a CDN and configure
your project to serve them from that location.
AWS has an article on deploying [React with SSR](https://aws.amazon.com/blogs/compute/building-server-side-rendering-for-react-in-aws-lambda/).

It's also pretty easy to set up edge compute with Lambda@Edge,
which should improve latency.
