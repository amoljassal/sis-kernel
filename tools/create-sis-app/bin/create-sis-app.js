#!/usr/bin/env node

const { program } = require('commander');
const chalk = require('chalk');
const inquirer = require('inquirer');
const fs = require('fs-extra');
const path = require('path');
const ora = require('ora');
const validateProjectName = require('validate-npm-package-name');

const TEMPLATES = {
  'react-saas': {
    name: 'React SaaS Application',
    description: 'Full-stack React app with authentication, billing, and analytics',
    tech: ['React', 'TypeScript', 'Vite', 'Tailwind', 'Stripe']
  },
  'vue-saas': {
    name: 'Vue SaaS Application', 
    description: 'Vue.js app with complete SaaS infrastructure',
    tech: ['Vue 3', 'TypeScript', 'Vite', 'Tailwind', 'Stripe']
  },
  'electron-app': {
    name: 'Electron Desktop Application',
    description: 'Cross-platform desktop app with SIS integration',
    tech: ['Electron', 'React', 'TypeScript', 'Auto-updater']
  },
  'api-service': {
    name: 'Node.js API Service',
    description: 'RESTful API service with database and authentication',
    tech: ['Node.js', 'Express', 'TypeScript', 'PostgreSQL', 'JWT']
  }
};

const CATEGORIES = {
  'productivity': 'Productivity & Collaboration',
  'development': 'Developer Tools',
  'design': 'Design & Creative',
  'ai-ml': 'AI & Machine Learning',
  'hardware': 'Hardware & IoT',
  'security': 'Security & Privacy',
  'finance': 'Finance & Business',
  'education': 'Education & Learning',
  'other': 'Other'
};

const REVENUE_MODELS = {
  'subscription': 'Monthly/Annual Subscription',
  'usage-based': 'Pay-per-use/Credits',
  'one-time': 'One-time Purchase',
  'freemium': 'Free with Premium Features',
  'marketplace': 'Transaction Fees',
  'free': 'Free (Open Source)'
};

program
  .name('create-sis-app')
  .description('Create a new SIS ecosystem application')
  .version('1.0.0')
  .argument('[project-name]', 'name of the project')
  .option('-t, --template <template>', 'specify template to use')
  .option('-y, --yes', 'use default configuration')
  .action(async (projectName, options) => {
    console.log(chalk.cyan('🚀 Welcome to SIS App Creator\n'));
    
    try {
      // Get project configuration
      const config = await getProjectConfig(projectName, options);
      
      // Create project
      await createProject(config);
      
      console.log(chalk.green('\n✅ SIS app created successfully!'));
      console.log(chalk.yellow('\n📋 Next steps:'));
      console.log(chalk.gray(`  cd ${config.name}`));
      console.log(chalk.gray('  npm install'));
      console.log(chalk.gray('  npm run dev'));
      console.log(chalk.gray('\n📖 Documentation: https://docs.sis.dev'));
      
    } catch (error) {
      console.error(chalk.red('\n❌ Error creating SIS app:'), error.message);
      process.exit(1);
    }
  });

async function getProjectConfig(projectName, options) {
  const questions = [];
  
  // Project name
  if (!projectName) {
    questions.push({
      type: 'input',
      name: 'name',
      message: 'Project name:',
      validate: (input) => {
        const validation = validateProjectName(input);
        return validation.validForNewPackages || validation.errors?.[0] || validation.warnings?.[0];
      }
    });
  }
  
  if (!options.yes && !options.template) {
    // Template selection
    questions.push({
      type: 'list',
      name: 'template',
      message: 'Choose a template:',
      choices: Object.entries(TEMPLATES).map(([key, template]) => ({
        name: `${template.name} - ${template.description}`,
        value: key,
        short: template.name
      }))
    });
    
    // App category
    questions.push({
      type: 'list',
      name: 'category',
      message: 'App category:',
      choices: Object.entries(CATEGORIES).map(([key, name]) => ({
        name,
        value: key
      }))
    });
    
    // Revenue model
    questions.push({
      type: 'list',
      name: 'revenueModel',
      message: 'Revenue model:',
      choices: Object.entries(REVENUE_MODELS).map(([key, name]) => ({
        name,
        value: key
      }))
    });
    
    // Additional features
    questions.push({
      type: 'checkbox',
      name: 'features',
      message: 'Select features:',
      choices: [
        { name: 'Authentication (Auth0)', value: 'auth', checked: true },
        { name: 'Payment processing (Stripe)', value: 'billing', checked: true },
        { name: 'Analytics (Mixpanel)', value: 'analytics', checked: true },
        { name: 'PWA support', value: 'pwa', checked: true },
        { name: 'Electron wrapper', value: 'electron', checked: false },
        { name: 'Database integration', value: 'database', checked: true },
        { name: 'Real-time features (WebSocket)', value: 'realtime', checked: false },
        { name: 'Email system', value: 'email', checked: true },
        { name: 'File uploads', value: 'uploads', checked: false },
        { name: 'Internationalization', value: 'i18n', checked: false }
      ]
    });
    
    // Description
    questions.push({
      type: 'input',
      name: 'description',
      message: 'App description:',
      default: 'A new SIS ecosystem application'
    });
    
    // Target launch
    questions.push({
      type: 'list',
      name: 'targetLaunch',
      message: 'Target launch timeline:',
      choices: [
        { name: '30 days (MVP)', value: '30d' },
        { name: '60 days (Beta)', value: '60d' },
        { name: '90 days (Full launch)', value: '90d' },
        { name: '6 months (Complex app)', value: '6m' }
      ]
    });
  }
  
  const answers = await inquirer.prompt(questions);
  
  // Merge with defaults
  const config = {
    name: projectName || answers.name,
    template: options.template || answers.template || 'react-saas',
    category: answers.category || 'other',
    revenueModel: answers.revenueModel || 'subscription',
    features: answers.features || ['auth', 'billing', 'analytics', 'pwa', 'database', 'email'],
    description: answers.description || 'A new SIS ecosystem application',
    targetLaunch: answers.targetLaunch || '60d',
    ...answers
  };
  
  return config;
}

async function createProject(config) {
  const spinner = ora('Creating SIS app...').start();
  
  try {
    const projectPath = path.resolve(config.name);
    
    // Check if directory exists
    if (await fs.pathExists(projectPath)) {
      throw new Error(`Directory ${config.name} already exists`);
    }
    
    // Create directory
    await fs.ensureDir(projectPath);
    
    // Copy template
    spinner.text = 'Copying template files...';
    const templatePath = path.join(__dirname, '..', 'templates', config.template);
    await fs.copy(templatePath, projectPath);
    
    // Generate configuration files
    spinner.text = 'Generating configuration...';
    await generatePackageJson(projectPath, config);
    await generateSISConfig(projectPath, config);
    await generateEnvironmentFiles(projectPath, config);
    await generateDocumentation(projectPath, config);
    
    // Process template variables
    spinner.text = 'Processing template variables...';
    await processTemplateVariables(projectPath, config);
    
    // Generate feature-specific files
    if (config.features.includes('auth')) {
      await generateAuthConfig(projectPath, config);
    }
    
    if (config.features.includes('billing')) {
      await generateBillingConfig(projectPath, config);
    }
    
    if (config.features.includes('analytics')) {
      await generateAnalyticsConfig(projectPath, config);
    }
    
    if (config.features.includes('pwa')) {
      await generatePWAConfig(projectPath, config);
    }
    
    // Create deployment configs
    spinner.text = 'Creating deployment configurations...';
    await generateDeploymentConfigs(projectPath, config);
    
    spinner.succeed('SIS app created successfully');
    
  } catch (error) {
    spinner.fail('Failed to create SIS app');
    throw error;
  }
}

async function generatePackageJson(projectPath, config) {
  const template = TEMPLATES[config.template];
  
  const packageJson = {
    name: config.name,
    version: '0.1.0',
    description: config.description,
    private: true,
    type: 'module',
    scripts: {
      dev: 'vite',
      build: 'vite build',
      preview: 'vite preview',
      test: 'vitest',
      'test:ui': 'vitest --ui',
      'test:coverage': 'vitest --coverage',
      lint: 'eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0',
      'lint:fix': 'eslint . --ext ts,tsx --fix',
      'type-check': 'tsc --noEmit',
      'deploy:staging': 'vercel --target staging',
      'deploy:production': 'vercel --prod',
      'analyze': 'npm run build && npx vite-bundle-analyzer dist/stats.html'
    },
    dependencies: {
      'react': '^18.2.0',
      'react-dom': '^18.2.0',
      'react-router-dom': '^6.8.0',
      '@tanstack/react-query': '^4.24.6',
      'zustand': '^4.3.6',
      'axios': '^1.3.4'
    },
    devDependencies: {
      '@types/react': '^18.0.28',
      '@types/react-dom': '^18.0.11',
      '@typescript-eslint/eslint-plugin': '^5.54.0',
      '@typescript-eslint/parser': '^5.54.0',
      '@vitejs/plugin-react': '^3.1.0',
      'eslint': '^8.35.0',
      'eslint-plugin-react-hooks': '^4.6.0',
      'eslint-plugin-react-refresh': '^0.3.4',
      'typescript': '^4.9.5',
      'vite': '^4.1.4',
      'vitest': '^0.28.5',
      '@vitest/ui': '^0.28.5',
      '@vitest/coverage-c8': '^0.28.5'
    },
    sis: {
      category: config.category,
      revenueModel: config.revenueModel,
      features: config.features,
      targetLaunch: config.targetLaunch,
      template: config.template,
      createdAt: new Date().toISOString()
    }
  };
  
  // Add feature-specific dependencies
  if (config.features.includes('auth')) {
    packageJson.dependencies['@auth0/auth0-react'] = '^2.0.0';
  }
  
  if (config.features.includes('billing')) {
    packageJson.dependencies['@stripe/stripe-js'] = '^1.46.0';
    packageJson.dependencies['@stripe/react-stripe-js'] = '^1.16.4';
  }
  
  if (config.features.includes('analytics')) {
    packageJson.dependencies['mixpanel-browser'] = '^2.45.0';
  }
  
  if (config.features.includes('pwa')) {
    packageJson.devDependencies['vite-plugin-pwa'] = '^0.14.4';
  }
  
  if (config.features.includes('database')) {
    packageJson.dependencies['@supabase/supabase-js'] = '^2.8.0';
  }
  
  if (config.features.includes('realtime')) {
    packageJson.dependencies['socket.io-client'] = '^4.6.1';
  }
  
  await fs.writeJSON(path.join(projectPath, 'package.json'), packageJson, { spaces: 2 });
}

async function generateSISConfig(projectPath, config) {
  const sisConfig = {
    $schema: 'https://schema.sis.dev/app-config.json',
    version: '1.0',
    app: {
      id: config.name,
      name: config.name.replace(/-/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
      description: config.description,
      category: config.category,
      version: '0.1.0',
      author: 'SIS Developer',
      license: 'MIT'
    },
    deployment: {
      web: {
        enabled: true,
        domain: `${config.name}.sis.dev`,
        cdn: true,
        ssl: true
      },
      pwa: {
        enabled: config.features.includes('pwa'),
        manifest: './public/manifest.json',
        serviceWorker: './src/sw.ts'
      },
      electron: {
        enabled: config.features.includes('electron'),
        platforms: ['win32', 'darwin', 'linux']
      }
    },
    revenue: {
      model: config.revenueModel,
      provider: config.features.includes('billing') ? 'stripe' : null,
      tiers: config.revenueModel === 'subscription' ? [
        {
          name: 'Free',
          price: 0,
          features: ['basic_features']
        },
        {
          name: 'Pro',
          price: 29,
          features: ['basic_features', 'advanced_features']
        },
        {
          name: 'Enterprise',
          price: 'custom',
          features: ['all_features', 'priority_support']
        }
      ] : []
    },
    integration: {
      level: 0,
      kernel_apis: [],
      permissions: [],
      target_level: 3,
      planned_date: null
    },
    analytics: {
      enabled: config.features.includes('analytics'),
      provider: 'mixpanel',
      events: ['app_started', 'feature_used', 'conversion'],
      metrics: ['dau', 'retention', 'revenue']
    },
    development: {
      target_launch: config.targetLaunch,
      milestones: generateMilestones(config.targetLaunch),
      team_size: 1,
      budget_estimate: estimateBudget(config)
    }
  };
  
  await fs.writeJSON(path.join(projectPath, 'sis.config.json'), sisConfig, { spaces: 2 });
}

async function generateEnvironmentFiles(projectPath, config) {
  const envExample = {
    // Authentication
    ...(config.features.includes('auth') && {
      'VITE_AUTH0_DOMAIN': 'your-auth0-domain.auth0.com',
      'VITE_AUTH0_CLIENT_ID': 'your-auth0-client-id',
      'VITE_AUTH0_AUDIENCE': 'your-api-identifier'
    }),
    
    // Billing
    ...(config.features.includes('billing') && {
      'VITE_STRIPE_PUBLISHABLE_KEY': 'pk_test_...',
      'STRIPE_SECRET_KEY': 'sk_test_...',
      'STRIPE_WEBHOOK_SECRET': 'whsec_...'
    }),
    
    // Analytics
    ...(config.features.includes('analytics') && {
      'VITE_MIXPANEL_PROJECT_TOKEN': 'your-mixpanel-token'
    }),
    
    // Database
    ...(config.features.includes('database') && {
      'VITE_SUPABASE_URL': 'https://your-project.supabase.co',
      'VITE_SUPABASE_ANON_KEY': 'your-anon-key',
      'SUPABASE_SERVICE_ROLE_KEY': 'your-service-role-key'
    }),
    
    // General
    'VITE_APP_NAME': config.name,
    'VITE_APP_URL': `https://${config.name}.sis.dev`,
    'VITE_API_URL': `https://api.${config.name}.sis.dev`
  };
  
  // Create .env.example
  const envExampleContent = Object.entries(envExample)
    .map(([key, value]) => `${key}=${value}`)
    .join('\n');
    
  await fs.writeFile(path.join(projectPath, '.env.example'), envExampleContent);
  
  // Create .env.local (empty)
  await fs.writeFile(path.join(projectPath, '.env.local'), '# Add your local environment variables here\n');
}

async function generateDocumentation(projectPath, config) {
  const readme = `# ${config.name}

${config.description}

## 🚀 Quick Start

\`\`\`bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Deploy to staging
npm run deploy:staging

# Deploy to production
npm run deploy:production
\`\`\`

## 📋 Features

${config.features.map(feature => `- ✅ ${feature.charAt(0).toUpperCase() + feature.slice(1).replace('-', ' ')}`).join('\n')}

## 🏗️ Architecture

This is a SIS ecosystem application built with:

${TEMPLATES[config.template].tech.map(tech => `- ${tech}`).join('\n')}

## 💰 Revenue Model

**Model:** ${REVENUE_MODELS[config.revenueModel]}

## 📈 Roadmap

**Target Launch:** ${config.targetLaunch === '30d' ? '30 days (MVP)' : 
                      config.targetLaunch === '60d' ? '60 days (Beta)' : 
                      config.targetLaunch === '90d' ? '90 days (Full launch)' : '6 months (Complex app)'}

## 🔧 Development

### Environment Setup

1. Copy \`.env.example\` to \`.env.local\`
2. Fill in your API keys and configuration
3. Run \`npm install\`
4. Start development with \`npm run dev\`

### Testing

\`\`\`bash
# Run tests
npm test

# Run tests with UI
npm run test:ui

# Run tests with coverage
npm run test:coverage
\`\`\`

### Deployment

This app is configured for deployment on:
- **Web:** Vercel/Netlify
- **Database:** Supabase/PlanetScale
- **CDN:** Cloudflare
- **Analytics:** Mixpanel

## 📚 Documentation

- [SIS Ecosystem Docs](https://docs.sis.dev)
- [App Development Guide](https://docs.sis.dev/apps)
- [Integration Guide](https://docs.sis.dev/integration)

## 🛠️ SIS Integration

This app is designed for eventual integration with the SIS operating system:

- **Current Level:** 0 (Standalone Web App)
- **Target Level:** 3 (Kernel-Aware Application)
- **Integration Timeline:** TBD

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details

---

Built with ❤️ for the SIS Ecosystem
`;

  await fs.writeFile(path.join(projectPath, 'README.md'), readme);
}

function generateMilestones(targetLaunch) {
  const milestones = {
    '30d': [
      { week: 1, goal: 'MVP development', deliverable: 'Core features implemented' },
      { week: 2, goal: 'Testing & polish', deliverable: 'Beta version ready' },
      { week: 3, goal: 'Deployment setup', deliverable: 'Production environment' },
      { week: 4, goal: 'Launch preparation', deliverable: 'Public launch' }
    ],
    '60d': [
      { week: 2, goal: 'Core development', deliverable: 'Basic functionality' },
      { week: 4, goal: 'Feature completion', deliverable: 'All features implemented' },
      { week: 6, goal: 'Beta testing', deliverable: 'User feedback integration' },
      { week: 8, goal: 'Production launch', deliverable: 'Public availability' }
    ],
    '90d': [
      { week: 3, goal: 'Foundation', deliverable: 'Core architecture' },
      { week: 6, goal: 'Feature development', deliverable: 'MVP features' },
      { week: 9, goal: 'Beta version', deliverable: 'Beta release' },
      { week: 12, goal: 'Production launch', deliverable: 'Full launch' }
    ],
    '6m': [
      { month: 1, goal: 'Planning & setup', deliverable: 'Architecture & design' },
      { month: 2, goal: 'Core development', deliverable: 'Basic functionality' },
      { month: 3, goal: 'Feature completion', deliverable: 'All features' },
      { month: 4, goal: 'Testing & optimization', deliverable: 'Performance tuning' },
      { month: 5, goal: 'Beta testing', deliverable: 'User validation' },
      { month: 6, goal: 'Launch', deliverable: 'Production release' }
    ]
  };
  
  return milestones[targetLaunch] || milestones['60d'];
}

function estimateBudget(config) {
  let budget = 0;
  
  // Base development cost
  const timelines = { '30d': 5000, '60d': 10000, '90d': 15000, '6m': 30000 };
  budget += timelines[config.targetLaunch] || 10000;
  
  // Feature costs
  const featureCosts = {
    auth: 500,
    billing: 1000,
    analytics: 300,
    pwa: 500,
    electron: 2000,
    database: 800,
    realtime: 1500,
    email: 400,
    uploads: 600,
    i18n: 1000
  };
  
  config.features.forEach(feature => {
    budget += featureCosts[feature] || 0;
  });
  
  // Template complexity
  const templateCosts = {
    'react-saas': 0,
    'vue-saas': 500,
    'electron-app': 2000,
    'api-service': 1000
  };
  
  budget += templateCosts[config.template] || 0;
  
  return budget;
}

// Additional helper functions for generating feature-specific configs
async function generateAuthConfig(projectPath, config) {
  // Generate auth configuration files
  const authConfig = {
    domain: process.env.VITE_AUTH0_DOMAIN,
    clientId: process.env.VITE_AUTH0_CLIENT_ID,
    audience: process.env.VITE_AUTH0_AUDIENCE,
    redirectUri: `${process.env.VITE_APP_URL}/callback`,
    scope: 'openid profile email',
    useRefreshTokens: true,
    cacheLocation: 'localstorage'
  };
  
  await fs.writeJSON(path.join(projectPath, 'src/config/auth.json'), authConfig, { spaces: 2 });
}

async function generateBillingConfig(projectPath, config) {
  // Generate Stripe configuration
  const billingConfig = {
    publishableKey: process.env.VITE_STRIPE_PUBLISHABLE_KEY,
    currency: 'usd',
    plans: [
      {
        id: 'free',
        name: 'Free',
        price: 0,
        interval: null,
        features: ['basic_features']
      },
      {
        id: 'pro_monthly',
        name: 'Pro',
        price: 2900, // $29.00 in cents
        interval: 'month',
        features: ['basic_features', 'advanced_features']
      }
    ]
  };
  
  await fs.writeJSON(path.join(projectPath, 'src/config/billing.json'), billingConfig, { spaces: 2 });
}

async function generateAnalyticsConfig(projectPath, config) {
  const analyticsConfig = {
    mixpanel: {
      token: process.env.VITE_MIXPANEL_PROJECT_TOKEN,
      debug: process.env.NODE_ENV === 'development',
      track_pageview: true,
      persistence: 'localStorage'
    },
    events: {
      app_launched: 'App Launched',
      user_registered: 'User Registered',
      subscription_started: 'Subscription Started',
      feature_used: 'Feature Used'
    }
  };
  
  await fs.writeJSON(path.join(projectPath, 'src/config/analytics.json'), analyticsConfig, { spaces: 2 });
}

async function generatePWAConfig(projectPath, config) {
  const manifest = {
    name: config.name.replace(/-/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
    short_name: config.name,
    description: config.description,
    start_url: '/',
    display: 'standalone',
    theme_color: '#0066cc',
    background_color: '#ffffff',
    icons: [
      {
        src: '/icons/icon-192x192.png',
        sizes: '192x192',
        type: 'image/png'
      },
      {
        src: '/icons/icon-512x512.png',
        sizes: '512x512',
        type: 'image/png'
      }
    ]
  };
  
  await fs.writeJSON(path.join(projectPath, 'public/manifest.json'), manifest, { spaces: 2 });
}

async function generateDeploymentConfigs(projectPath, config) {
  // Vercel config
  const vercelConfig = {
    name: config.name,
    version: 2,
    builds: [
      { src: 'package.json', use: '@vercel/static-build' }
    ],
    routes: [
      { src: '/(.*)', dest: '/index.html' }
    ],
    env: {
      VITE_APP_NAME: config.name,
      VITE_APP_URL: `https://${config.name}.sis.dev`
    }
  };
  
  await fs.writeJSON(path.join(projectPath, 'vercel.json'), vercelConfig, { spaces: 2 });
  
  // GitHub Actions
  const githubWorkflow = `name: CI/CD

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18
      - run: npm ci
      - run: npm run test
      - run: npm run build

  deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3
      - uses: vercel/action@v20
        with:
          vercel-token: \${{ secrets.VERCEL_TOKEN }}
          vercel-org-id: \${{ secrets.ORG_ID }}
          vercel-project-id: \${{ secrets.PROJECT_ID }}
`;
  
  await fs.ensureDir(path.join(projectPath, '.github/workflows'));
  await fs.writeFile(path.join(projectPath, '.github/workflows/ci.yml'), githubWorkflow);
}

async function processTemplateVariables(projectPath, config) {
  // Process template files and replace variables
  const files = await fs.readdir(projectPath, { recursive: true });
  
  for (const file of files) {
    const filePath = path.join(projectPath, file);
    const stats = await fs.stat(filePath);
    
    if (stats.isFile() && (file.endsWith('.ts') || file.endsWith('.tsx') || file.endsWith('.js') || file.endsWith('.jsx'))) {
      let content = await fs.readFile(filePath, 'utf-8');
      
      // Replace template variables
      content = content
        .replace(/\{\{APP_NAME\}\}/g, config.name)
        .replace(/\{\{APP_DESCRIPTION\}\}/g, config.description)
        .replace(/\{\{APP_CATEGORY\}\}/g, config.category)
        .replace(/\{\{REVENUE_MODEL\}\}/g, config.revenueModel);
        
      await fs.writeFile(filePath, content);
    }
  }
}

if (require.main === module) {
  program.parse();
}

module.exports = { createProject, generatePackageJson, generateSISConfig };