# Create SIS App

A CLI tool for creating new SIS ecosystem applications with standardized templates, configuration, and best practices.

## 🚀 Quick Start

```bash
# Create a new SIS app
npx create-sis-app my-awesome-app

# Or use globally installed version
npm install -g create-sis-app
create-sis-app my-awesome-app
```

## 📋 Features

### Automated Setup
- ✅ **Project scaffolding** with best practices
- ✅ **TypeScript configuration** with strict settings
- ✅ **Tailwind CSS** with SIS design system
- ✅ **Testing setup** with Vitest and Playwright
- ✅ **Deployment configuration** for Vercel/Netlify
- ✅ **CI/CD pipeline** with GitHub Actions

### SaaS Infrastructure
- ✅ **Authentication** integration (Auth0/Clerk)
- ✅ **Billing & subscriptions** (Stripe/Paddle) 
- ✅ **Analytics** tracking (Mixpanel/Segment)
- ✅ **Database** integration (Supabase/PlanetScale)
- ✅ **Email** system (SendGrid/Resend)
- ✅ **File uploads** (AWS S3/Cloudflare R2)

### Progressive Enhancement
- ✅ **PWA support** with service workers
- ✅ **Electron wrapper** for desktop apps
- ✅ **Performance optimization** with lazy loading
- ✅ **SEO optimization** with meta management
- ✅ **Error tracking** (Sentry)
- ✅ **Monitoring** (DataDog/New Relic)

## 🎨 Available Templates

### React SaaS (`react-saas`)
Full-stack React application with complete SaaS infrastructure
- **Tech Stack:** React 18, TypeScript, Vite, Tailwind CSS
- **Features:** Auth, billing, analytics, PWA, database
- **Best For:** Most SaaS applications

### Vue SaaS (`vue-saas`) 
Vue.js application with SaaS capabilities
- **Tech Stack:** Vue 3, TypeScript, Vite, Tailwind CSS
- **Features:** Auth, billing, analytics, PWA, database
- **Best For:** Teams preferring Vue ecosystem

### Electron App (`electron-app`)
Cross-platform desktop application
- **Tech Stack:** Electron, React, TypeScript, Auto-updater
- **Features:** Native OS integration, auto-updates, system tray
- **Best For:** Desktop-first applications

### API Service (`api-service`)
Backend API service with authentication and database
- **Tech Stack:** Node.js, Express, TypeScript, PostgreSQL
- **Features:** JWT auth, database ORM, API documentation
- **Best For:** Backend services, microservices

## 🏗️ Interactive Setup

The CLI will guide you through configuration:

```bash
npx create-sis-app
```

### Configuration Options

#### App Category
- **Productivity & Collaboration** - Team tools, project management
- **Developer Tools** - IDEs, build tools, debugging
- **Design & Creative** - Design tools, content creation  
- **AI & Machine Learning** - ML platforms, data tools
- **Hardware & IoT** - Hardware control, device management
- **Security & Privacy** - Security tools, compliance
- **Finance & Business** - Fintech, business intelligence
- **Education & Learning** - EdTech, training platforms

#### Revenue Models
- **Subscription** - Monthly/annual recurring revenue
- **Usage-based** - Pay-per-use, credits, metered billing
- **One-time** - Single purchase, lifetime license
- **Freemium** - Free tier with premium upgrades
- **Marketplace** - Transaction fees, commissions
- **Free** - Open source, community-driven

#### Features Selection
- **Authentication** - User login, social auth, SSO
- **Billing** - Subscription management, payment processing
- **Analytics** - User tracking, conversion funnels
- **PWA** - Progressive web app, offline support
- **Database** - Data persistence, real-time sync
- **Email** - Transactional emails, notifications
- **File Uploads** - Asset management, CDN integration
- **Real-time** - WebSocket, live collaboration
- **Internationalization** - Multi-language support

## 📁 Generated Project Structure

```
my-app/
├── public/
│   ├── icons/                 # PWA icons
│   ├── manifest.json         # PWA manifest
│   └── index.html
├── src/
│   ├── components/           # Reusable components
│   │   ├── ui/              # UI primitives
│   │   └── Layout.tsx
│   ├── pages/               # Route components
│   ├── lib/                 # Utilities
│   │   ├── auth.ts         # Authentication logic
│   │   ├── billing.ts      # Billing integration
│   │   ├── analytics.ts    # Analytics tracking
│   │   └── api.ts          # API client
│   ├── hooks/              # Custom React hooks
│   ├── contexts/           # React contexts
│   ├── types/              # TypeScript types
│   ├── styles/             # Global styles
│   └── App.tsx
├── tests/                  # Test files
├── .env.example           # Environment template
├── .env.local            # Local environment
├── sis.config.json       # SIS app configuration
├── package.json
├── tailwind.config.js
├── tsconfig.json
├── vite.config.ts
├── vercel.json           # Deployment config
└── README.md
```

## ⚙️ Configuration Files

### SIS App Configuration (`sis.config.json`)
```json
{
  "$schema": "https://schema.sis.dev/app-config.json",
  "app": {
    "id": "my-app",
    "name": "My App", 
    "category": "productivity",
    "version": "0.1.0"
  },
  "revenue": {
    "model": "subscription",
    "tiers": [
      { "name": "Free", "price": 0 },
      { "name": "Pro", "price": 29 }
    ]
  },
  "integration": {
    "level": 0,
    "target_level": 3,
    "kernel_apis": []
  }
}
```

### Environment Configuration
```bash
# Authentication (Auth0)
VITE_AUTH0_DOMAIN=your-domain.auth0.com
VITE_AUTH0_CLIENT_ID=your-client-id

# Billing (Stripe)
VITE_STRIPE_PUBLISHABLE_KEY=pk_test_...
STRIPE_SECRET_KEY=sk_test_...

# Analytics (Mixpanel)
VITE_MIXPANEL_PROJECT_TOKEN=your-token

# Database (Supabase)  
VITE_SUPABASE_URL=https://your-project.supabase.co
VITE_SUPABASE_ANON_KEY=your-anon-key
```

## 🚀 Development Workflow

### 1. Development
```bash
cd my-app
npm install
npm run dev
```

### 2. Testing
```bash
npm test              # Unit tests
npm run test:ui       # Test UI
npm run test:e2e      # End-to-end tests
```

### 3. Building
```bash
npm run build         # Production build
npm run preview       # Preview build
```

### 4. Deployment
```bash
npm run deploy:staging    # Deploy to staging
npm run deploy:production # Deploy to production
```

## 📊 Built-in Analytics

### Automatic Event Tracking
- **App launched** - User opens application
- **User registered** - Account creation
- **Subscription started** - Billing conversion
- **Feature used** - Core functionality usage
- **Error occurred** - Application errors

### Custom Events
```typescript
import { analytics } from './lib/analytics';

// Track custom events
analytics.track('feature_clicked', {
  feature_name: 'advanced_settings',
  user_plan: 'pro'
});

// Identify users
analytics.identify('user-123', {
  email: 'user@example.com',
  plan: 'pro'
});
```

## 🔐 Security Best Practices

### Built-in Security
- **HTTPS enforcement** for all environments
- **CSP headers** for XSS protection  
- **CSRF protection** for forms
- **Dependency scanning** with npm audit
- **Secret detection** in CI/CD
- **Environment isolation** dev/staging/prod

### Authentication Security
- **Secure JWT storage** in httpOnly cookies
- **Token refresh** handling
- **Multi-factor authentication** support
- **Session management** with expiration
- **Social auth** with OAuth2/OIDC

## 📈 Performance Optimization

### Automatic Optimizations
- **Code splitting** with dynamic imports
- **Tree shaking** for minimal bundles
- **Asset compression** with gzip/brotli
- **Image optimization** with modern formats
- **Lazy loading** for components and routes
- **Service worker** caching strategies

### Performance Monitoring
- **Web Vitals** tracking
- **Bundle analysis** reports
- **Lighthouse** CI integration
- **Error rate** monitoring
- **Performance budgets** enforcement

## 🔧 Customization

### Extending Templates
```bash
# Create custom template
mkdir templates/my-custom-template
cp -r templates/react-saas/* templates/my-custom-template/

# Modify template files with {{VARIABLES}}
# Update create-sis-app.js to include new template
```

### Adding Features
```typescript
// Add to feature configuration
const features = {
  myFeature: {
    name: 'My Custom Feature',
    dependencies: ['some-package'],
    setup: async (projectPath, config) => {
      // Custom setup logic
    }
  }
};
```

## 📚 Integration Guides

### SIS Kernel Integration
Prepare your app for eventual SIS OS integration:

1. **Define kernel APIs needed**
2. **Design permission model**
3. **Plan native compilation**
4. **Set integration timeline**

### Third-party Services
Pre-configured integrations:

- **Auth0/Clerk** - Authentication
- **Stripe/Paddle** - Payments
- **Mixpanel/Segment** - Analytics  
- **Supabase/PlanetScale** - Database
- **Vercel/Netlify** - Hosting
- **Sentry** - Error tracking
- **SendGrid/Resend** - Email

## 🤝 Contributing

### Development Setup
```bash
git clone https://github.com/sis-ecosystem/create-sis-app
cd create-sis-app
npm install
npm link

# Test locally
create-sis-app test-app
```

### Adding Templates
1. Create template in `templates/`
2. Add to `TEMPLATES` object
3. Update CLI prompts
4. Add tests
5. Update documentation

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🔗 Resources

- [SIS Ecosystem Documentation](https://docs.sis.dev)
- [App Development Guide](https://docs.sis.dev/apps)
- [Integration Patterns](https://docs.sis.dev/integration)
- [Community Discord](https://discord.gg/sis-dev)
- [GitHub Discussions](https://github.com/sis-ecosystem/create-sis-app/discussions)

---

**Create SIS App** - Rapidly scaffold SIS ecosystem applications with enterprise-grade infrastructure and best practices built-in.