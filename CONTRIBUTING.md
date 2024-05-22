# Contributing

Thanks for taking the time to submit code to `reqTUI` if you're reading this!
We love having new contributors and love seeing the community come around and
help making it better.

There are opportunities to contribute to `reqTUI` at any level. It doesn't
matter if you are just getting started with Rust or are the most weathered
expert, we can use your help.

**No contribution is too small and all contributions are valued.**

## Pull Requests

Pull Requests are the way concrete changes are made to the project, Even tiny
pull requests are greatly appreciated

### Tests

If the change being proposed alters code, it is either adding new functionality
to a crate or it is fixing existing, broken functionality. In both of these
cases, the pull request should include one or more tests to ensure that the
project does not regress in the future.

### Commits

It is a recommended best practice to keep your changes as logically grouped as
possible within individual commits. There is no limit to the number of commits
any single Pull Request may have, and many contributors find it easier to review
changes that are split across multiple commits.

#### Commit message guidelines

A good commit message should describe what changed and why, and also follow the
[conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) guidelines

1. Add a meaningful and short description on the first line of your commit
2. You can opt to add a more detailed description, in that case, keep the second
   line empty, and use any other number of lines to detail your changes.
3. If your commit addresses an issue, its a good idea to mention it.

Examples:
`fix(42): line wrapping going out of bounds`
`feat: importing cURL commands`

### Opening the Pull Request

From within GitHub, opening a new Pull Request will present you with a
[template] that should be filled out. Please try to do your best at filling out
the details, but feel free to skip parts if you're not sure what to put.

[template]: .github/PULL_REQUEST_TEMPLATE.md

### Discuss and update

You will probably get feedback or requests for changes to your Pull Request.
This is a big part of the submission process so don't be discouraged! Some
contributors may sign off on the Pull Request right away, others may have
more detailed comments or feedback. This is a necessary part of the process
in order to evaluate whether the changes are correct and necessary.

### Be aware of the person behind the code

Be aware that *how* you communicate requests and reviews in your feedback can
have a significant impact on the success of the Pull Request. Yes, we may land
a particular change that makes `reqTUI` better, but the individual might just
not want to have anything to do with `reqTUI` ever again. The goal is not
just having good code.
