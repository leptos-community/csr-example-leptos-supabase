# Leptos-Supabase

Leptos-Supabase is a web application that uses the Leptos framework as the client and Supabase as the backend service. It demonstrates how to implement authentication with Google and email/password using Supabase.

You can check out the demo site which is public for a limited time:
https://mahdi739.github.io/leptos_supabase_example/

## Features

- Leptos is a lightweight and fast web framework that supports server-side rendering, routing, and state management.
- Supabase is an open source Firebase alternative that provides a suite of tools for building web applications, such as database, authentication, storage, and functions.
- Authentication with Google and email/password allows users to sign in securely and conveniently using their preferred method.

## Deploy

To deploy Leptos-Supabase, you need to:

- Clone this repository to your local machine.
- Create a Supabase project and get the following environment variables:
  - `APP_DATABASE_URL`: The REST API URL for your database, which is provided by your Supabase project. It should look like `[supabase_url]/rest/v1`.
  - `APP_API_KEY`: The API key from your Supabase project, which you can find in the Settings > API section of your Supabase dashboard.
  - `APP_SIGNUP_URL`: The URL for signing up users with email/password, which is provided by your Supabase project. It should look like `[supabase_url]/auth/v1/signup`.
  - `APP_GOOGLE_LOGIN_URL`: The URL for signing in users with Google, which is provided by your Supabase project. It should look like `[supabase_url]/auth/v1/authorize?provider=google&redirect_to=[redirect_url]`, where `[redirect_url]` is the encoded URL that the OAuth service will send the token to.
  - `APP_MANUAL_LOGIN_URL`: The URL for signing in users with email/password, which is provided by your Supabase project. It should look like `[supabase_url]/auth/v1/token?grant_type=password`. For this method, you need to set the redirect URL in the Settings > Auth > Settings section of your Supabase dashboard, not in the query parameters.
  - `APP_REFRESH_TOKEN_URL`: The URL for refreshing the user token, which is provided by your Supabase project. It should look like `[supabase_url]/auth/v1/token?grant_type=refresh_token`.
- Create a `.env` file in the root directory of your project and add the environment variables with their values.

