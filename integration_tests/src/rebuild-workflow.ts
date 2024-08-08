import { parse, stringify } from 'yaml';
import { readFileSync, writeFileSync } from 'fs';

const integrationTestsWorkflow = (name: string) => [
  {
    name: 'Upgrade docker compose to use v2',
    run: 'sudo curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose && sudo chmod +x /usr/local/bin/docker-compose',
  },
  {
    uses: 'actions/checkout@v4',
    with: {
      'fetch-depth': 1,
    },
  },
  {
    name: 'Setup node',
    uses: 'actions/setup-node@v4',
    with: {
      'node-version': '20.12.2',
    },
  },
  {
    name: 'Install Yarn',
    run: 'npm install -g yarn',
  },
  {
    name: 'Log in to Private Registry',
    uses: 'docker/login-action@v3',
    with: {
      username: '${{ secrets.DOCKER_USER }}',
      password: '${{ secrets.DOCKER_TOKEN }}',
    },
  },
  {
    name: 'Clean volumes',
    run: 'docker volume prune -f',
  },
  {
    name: 'Download images',
    run: 'cd integration_tests\nyarn build-images\n',
  },
  {
    name: 'Download artifacts',
    uses: 'actions/cache@v4',
    with: {
      path: 'artifacts',
      key: '${{ runner.os }}-${{ github.sha }}',
    },
  },
  {
    name: `Run test ${name}`,
    run: `cd integration_tests && yarn && yarn ${name}`,
  },
  {
    name: 'Cleanup resources',
    if: 'always()',
    run: 'docker stop -t0 $(docker ps -a -q) || true\ndocker container prune -f || true\ndocker volume rm $(docker volume ls -q) || true\n',
  },
];

const integrationWorkflow = (name: string) => ({
  name: `${name} Integration Tests`,
  needs: ['images-prepare', 'artifacts-prepare'],
  'runs-on': 'self-hosted',
  steps: integrationTestsWorkflow(name),
});

const packageJson = JSON.parse(
  readFileSync(__dirname + `/../package.json`).toString(),
);

const names = Object.keys(packageJson.scripts)
  .filter((name) => name.includes(':'))
  .filter((name) => name.includes('test'));

const workflow = parse(readFileSync(__dirname + `/tests.yml`).toString());
names.forEach((name) => {
  workflow.jobs[name.replace(/:/g, '-')] = integrationWorkflow(name);
});
writeFileSync(
  __dirname + `/../../.github/workflows/tests.yml`,
  stringify(workflow),
);
