/**
 * Hindi Language Support and Indian Localization
 * Phase 5A: Multi-language support for Indian market
 */

export interface LocalizedContent {
  key: string;
  english: string;
  hindi: string;
  romanized?: string; // For users who can't read Devanagari
  audioUrl?: string; // For pronunciation help
}

export interface RegionalSettings {
  language: 'en-IN' | 'hi-IN' | 'ta-IN' | 'te-IN' | 'mr-IN';
  dateFormat: 'DD/MM/YYYY' | 'MM/DD/YYYY';
  timeFormat: '12h' | '24h';
  numberFormat: 'indian' | 'international'; // 1,00,000 vs 100,000
  currency: 'INR';
  timezone: 'Asia/Kolkata';
  firstDayOfWeek: 'monday' | 'sunday';
  workingDays: string[];
  holidays: 'indian_national' | 'state_specific';
}

export const HINDI_TRANSLATIONS: Record<string, LocalizedContent> = {
  // Navigation and UI
  'nav.dashboard': {
    key: 'nav.dashboard',
    english: 'Dashboard',
    hindi: 'डैशबोर्ड',
    romanized: 'Dashboard'
  },
  'nav.designer': {
    key: 'nav.designer',
    english: 'Designer',
    hindi: 'डिजाइनर',
    romanized: 'Designer'
  },
  'nav.projects': {
    key: 'nav.projects',
    english: 'Projects',
    hindi: 'प्रोजेक्ट्स',
    romanized: 'Projects'
  },
  'nav.marketplace': {
    key: 'nav.marketplace',
    english: 'Marketplace',
    hindi: 'मार्केटप्लेस',
    romanized: 'Marketplace'
  },
  'nav.education': {
    key: 'nav.education',
    english: 'Education',
    hindi: 'शिक्षा',
    romanized: 'Shiksha'
  },
  'nav.settings': {
    key: 'nav.settings',
    english: 'Settings',
    hindi: 'सेटिंग्स',
    romanized: 'Settings'
  },

  // Technical Terms (Digital Electronics)
  'tech.logic_gate': {
    key: 'tech.logic_gate',
    english: 'Logic Gate',
    hindi: 'तर्क गेट',
    romanized: 'Tark Gate'
  },
  'tech.and_gate': {
    key: 'tech.and_gate',
    english: 'AND Gate',
    hindi: 'एंड गेट',
    romanized: 'AND Gate'
  },
  'tech.or_gate': {
    key: 'tech.or_gate',
    english: 'OR Gate',
    hindi: 'ओआर गेट',
    romanized: 'OR Gate'
  },
  'tech.not_gate': {
    key: 'tech.not_gate',
    english: 'NOT Gate',
    hindi: 'नॉट गेट',
    romanized: 'NOT Gate'
  },
  'tech.flip_flop': {
    key: 'tech.flip_flop',
    english: 'Flip Flop',
    hindi: 'फ्लिप फ्लॉप',
    romanized: 'Flip Flop'
  },
  'tech.multiplexer': {
    key: 'tech.multiplexer',
    english: 'Multiplexer',
    hindi: 'मल्टीप्लेक्सर',
    romanized: 'Multiplexer'
  },
  'tech.decoder': {
    key: 'tech.decoder',
    english: 'Decoder',
    hindi: 'डिकोडर',
    romanized: 'Decoder'
  },
  'tech.counter': {
    key: 'tech.counter',
    english: 'Counter',
    hindi: 'काउंटर',
    romanized: 'Counter'
  },
  'tech.register': {
    key: 'tech.register',
    english: 'Register',
    hindi: 'रजिस्टर',
    romanized: 'Register'
  },
  'tech.memory': {
    key: 'tech.memory',
    english: 'Memory',
    hindi: 'मेमोरी',
    romanized: 'Memory'
  },
  'tech.processor': {
    key: 'tech.processor',
    english: 'Processor',
    hindi: 'प्रोसेसर',
    romanized: 'Processor'
  },
  'tech.microcontroller': {
    key: 'tech.microcontroller',
    english: 'Microcontroller',
    hindi: 'माइक्रोकंट्रोलर',
    romanized: 'Microcontroller'
  },

  // Educational Content
  'edu.objective': {
    key: 'edu.objective',
    english: 'Learning Objective',
    hindi: 'सीखने का उद्देश्य',
    romanized: 'Seekhne ka Uddeshya'
  },
  'edu.prerequisite': {
    key: 'edu.prerequisite',
    english: 'Prerequisites',
    hindi: 'आवश्यक शर्तें',
    romanized: 'Aavashyak Shartein'
  },
  'edu.outcome': {
    key: 'edu.outcome',
    english: 'Learning Outcome',
    hindi: 'सीखने का परिणाम',
    romanized: 'Seekhne ka Parinaam'
  },
  'edu.assessment': {
    key: 'edu.assessment',
    english: 'Assessment',
    hindi: 'मूल्यांकन',
    romanized: 'Mulyaankan'
  },
  'edu.quiz': {
    key: 'edu.quiz',
    english: 'Quiz',
    hindi: 'प्रश्नोत्तरी',
    romanized: 'Prashnottari'
  },
  'edu.assignment': {
    key: 'edu.assignment',
    english: 'Assignment',
    hindi: 'असाइनमेंट',
    romanized: 'Assignment'
  },
  'edu.project': {
    key: 'edu.project',
    english: 'Project',
    hindi: 'प्रोजेक्ट',
    romanized: 'Project'
  },
  'edu.grade': {
    key: 'edu.grade',
    english: 'Grade',
    hindi: 'ग्रेड',
    romanized: 'Grade'
  },
  'edu.progress': {
    key: 'edu.progress',
    english: 'Progress',
    hindi: 'प्रगति',
    romanized: 'Pragati'
  },
  'edu.certificate': {
    key: 'edu.certificate',
    english: 'Certificate',
    hindi: 'प्रमाण पत्र',
    romanized: 'Pramaan Patra'
  },

  // Common Actions
  'action.create': {
    key: 'action.create',
    english: 'Create',
    hindi: 'बनाएं',
    romanized: 'Banaen'
  },
  'action.edit': {
    key: 'action.edit',
    english: 'Edit',
    hindi: 'संपादित करें',
    romanized: 'Sampadit Karen'
  },
  'action.delete': {
    key: 'action.delete',
    english: 'Delete',
    hindi: 'हटाएं',
    romanized: 'Hataen'
  },
  'action.save': {
    key: 'action.save',
    english: 'Save',
    hindi: 'सहेजें',
    romanized: 'Sahejein'
  },
  'action.cancel': {
    key: 'action.cancel',
    english: 'Cancel',
    hindi: 'रद्द करें',
    romanized: 'Radd Karen'
  },
  'action.submit': {
    key: 'action.submit',
    english: 'Submit',
    hindi: 'जमा करें',
    romanized: 'Jama Karen'
  },
  'action.download': {
    key: 'action.download',
    english: 'Download',
    hindi: 'डाउनलोड करें',
    romanized: 'Download Karen'
  },
  'action.upload': {
    key: 'action.upload',
    english: 'Upload',
    hindi: 'अपलोड करें',
    romanized: 'Upload Karen'
  },

  // Pricing and Billing
  'billing.subscription': {
    key: 'billing.subscription',
    english: 'Subscription',
    hindi: 'सब्स्क्रिप्शन',
    romanized: 'Subscription'
  },
  'billing.free_plan': {
    key: 'billing.free_plan',
    english: 'Free Plan',
    hindi: 'मुफ्त योजना',
    romanized: 'Muft Yojana'
  },
  'billing.student_plan': {
    key: 'billing.student_plan',
    english: 'Student Plan',
    hindi: 'छात्र योजना',
    romanized: 'Chhaatra Yojana'
  },
  'billing.pro_plan': {
    key: 'billing.pro_plan',
    english: 'Pro Plan',
    hindi: 'प्रो योजना',
    romanized: 'Pro Yojana'
  },
  'billing.enterprise_plan': {
    key: 'billing.enterprise_plan',
    english: 'Enterprise Plan',
    hindi: 'एंटरप्राइज योजना',
    romanized: 'Enterprise Yojana'
  },
  'billing.monthly': {
    key: 'billing.monthly',
    english: 'Monthly',
    hindi: 'मासिक',
    romanized: 'Maasik'
  },
  'billing.yearly': {
    key: 'billing.yearly',
    english: 'Yearly',
    hindi: 'वार्षिक',
    romanized: 'Vaarshik'
  },
  'billing.rupees': {
    key: 'billing.rupees',
    english: 'Rupees',
    hindi: 'रुपये',
    romanized: 'Rupaye'
  },

  // Error Messages
  'error.network': {
    key: 'error.network',
    english: 'Network connection error. Please check your internet connection.',
    hindi: 'नेटवर्क कनेक्शन त्रुटि। कृपया अपना इंटरनेट कनेक्शन जांचें।',
    romanized: 'Network connection error. Kripaya apna internet connection jaanchein.'
  },
  'error.validation': {
    key: 'error.validation',
    english: 'Please fill all required fields.',
    hindi: 'कृपया सभी आवश्यक फील्ड भरें।',
    romanized: 'Kripaya sabhi aavashyak field bharein.'
  },
  'error.permission': {
    key: 'error.permission',
    english: 'You do not have permission to perform this action.',
    hindi: 'आपके पास यह कार्य करने की अनुमति नहीं है।',
    romanized: 'Aapke paas yah kaarya karne ki anumati nahin hai.'
  },

  // Success Messages
  'success.saved': {
    key: 'success.saved',
    english: 'Changes saved successfully!',
    hindi: 'परिवर्तन सफलतापूर्वक सहेजे गए!',
    romanized: 'Parivartan safaltaapoorvak saheje gaye!'
  },
  'success.created': {
    key: 'success.created',
    english: 'Created successfully!',
    hindi: 'सफलतापूर्वक बनाया गया!',
    romanized: 'Safaltaapoorvak banaaya gaya!'
  },
  'success.deleted': {
    key: 'success.deleted',
    english: 'Deleted successfully!',
    hindi: 'सफलतापूर्वक हटा दिया गया!',
    romanized: 'Safaltaapoorvak hata diya gaya!'
  },

  // Competitive Exams
  'exam.gate': {
    key: 'exam.gate',
    english: 'GATE Preparation',
    hindi: 'गेट तैयारी',
    romanized: 'GATE Taiyari'
  },
  'exam.jee': {
    key: 'exam.jee',
    english: 'JEE Preparation',
    hindi: 'जेईई तैयारी',
    romanized: 'JEE Taiyari'
  },
  'exam.mock_test': {
    key: 'exam.mock_test',
    english: 'Mock Test',
    hindi: 'मॉक टेस्ट',
    romanized: 'Mock Test'
  },
  'exam.previous_year': {
    key: 'exam.previous_year',
    english: 'Previous Year Questions',
    hindi: 'पिछले साल के प्रश्न',
    romanized: 'Pichhle Saal ke Prashn'
  },

  // Placement and Career
  'career.placement': {
    key: 'career.placement',
    english: 'Placement Preparation',
    hindi: 'प्लेसमेंट तैयारी',
    romanized: 'Placement Taiyari'
  },
  'career.interview': {
    key: 'career.interview',
    english: 'Interview Preparation',
    hindi: 'साक्षात्कार तैयारी',
    romanized: 'Saakshaatkaar Taiyari'
  },
  'career.resume': {
    key: 'career.resume',
    english: 'Resume Building',
    hindi: 'रेज्यूमे बनाना',
    romanized: 'Resume Banaana'
  },
  'career.skills': {
    key: 'career.skills',
    english: 'Skill Development',
    hindi: 'कौशल विकास',
    romanized: 'Kaushal Vikaas'
  }
};

export class HindiLocalizationService {
  private currentLanguage: 'en-IN' | 'hi-IN' = 'en-IN';

  constructor() {
    this.detectUserLanguage();
  }

  /**
   * Get localized text for a given key
   */
  getText(key: string, options?: { 
    format?: 'devanagari' | 'romanized' | 'both';
    fallback?: string;
  }): string {
    const content = HINDI_TRANSLATIONS[key];
    
    if (!content) {
      return options?.fallback || key;
    }

    if (this.currentLanguage === 'en-IN') {
      return content.english;
    }

    switch (options?.format) {
      case 'romanized':
        return content.romanized || content.hindi;
      case 'both':
        return `${content.hindi} (${content.romanized || content.english})`;
      case 'devanagari':
      default:
        return content.hindi;
    }
  }

  /**
   * Format numbers in Indian style (1,00,000 instead of 100,000)
   */
  formatNumber(number: number, style: 'indian' | 'international' = 'indian'): string {
    if (style === 'international') {
      return number.toLocaleString('en-US');
    }

    // Indian number formatting
    return number.toLocaleString('en-IN');
  }

  /**
   * Format currency in Indian Rupees
   */
  formatCurrency(amount: number, options?: {
    showSymbol?: boolean;
    showDecimals?: boolean;
  }): string {
    const { showSymbol = true, showDecimals = false } = options || {};
    
    const formatted = this.formatNumber(amount, 'indian');
    const symbol = showSymbol ? '₹' : '';
    
    if (showDecimals) {
      return `${symbol}${formatted}.00`;
    }
    
    return `${symbol}${formatted}`;
  }

  /**
   * Format dates in Indian format (DD/MM/YYYY)
   */
  formatDate(date: Date, format?: 'short' | 'long' | 'relative'): string {
    const options: Intl.DateTimeFormatOptions = {
      timeZone: 'Asia/Kolkata'
    };

    switch (format) {
      case 'short':
        options.day = '2-digit';
        options.month = '2-digit';
        options.year = 'numeric';
        break;
      case 'long':
        options.day = 'numeric';
        options.month = 'long';
        options.year = 'numeric';
        break;
      case 'relative':
        return this.getRelativeDate(date);
      default:
        options.day = '2-digit';
        options.month = '2-digit';
        options.year = 'numeric';
    }

    if (this.currentLanguage === 'hi-IN') {
      return date.toLocaleDateString('hi-IN', options);
    }
    
    return date.toLocaleDateString('en-IN', options);
  }

  /**
   * Format time in Indian timezone
   */
  formatTime(date: Date, format?: '12h' | '24h'): string {
    const options: Intl.DateTimeFormatOptions = {
      timeZone: 'Asia/Kolkata',
      hour: '2-digit',
      minute: '2-digit',
      hour12: format === '12h' || format === undefined
    };

    if (this.currentLanguage === 'hi-IN') {
      return date.toLocaleTimeString('hi-IN', options);
    }
    
    return date.toLocaleTimeString('en-IN', options);
  }

  /**
   * Get user's preferred language settings
   */
  getUserSettings(): RegionalSettings {
    return {
      language: this.currentLanguage,
      dateFormat: 'DD/MM/YYYY',
      timeFormat: '12h',
      numberFormat: 'indian',
      currency: 'INR',
      timezone: 'Asia/Kolkata',
      firstDayOfWeek: 'monday',
      workingDays: ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'],
      holidays: 'indian_national'
    };
  }

  /**
   * Set user's language preference
   */
  setLanguage(language: 'en-IN' | 'hi-IN'): void {
    this.currentLanguage = language;
    localStorage.setItem('sis_language_preference', language);
    
    // Update HTML lang attribute
    document.documentElement.lang = language;
    
    // Update direction for Hindi (though Hindi is LTR)
    document.documentElement.dir = 'ltr';
  }

  /**
   * Get language-specific font recommendations
   */
  getFontRecommendations(): {
    primary: string;
    fallback: string[];
    webFont?: string;
  } {
    if (this.currentLanguage === 'hi-IN') {
      return {
        primary: 'Noto Sans Devanagari',
        fallback: ['Arial Unicode MS', 'Mangal', 'sans-serif'],
        webFont: 'https://fonts.googleapis.com/css2?family=Noto+Sans+Devanagari:wght@400;500;600;700&display=swap'
      };
    }

    return {
      primary: 'system-ui',
      fallback: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif']
    };
  }

  /**
   * Get educational terminology in both languages for better understanding
   */
  getEducationalTerms(): Record<string, { english: string; hindi: string; definition: string }> {
    return {
      logic_design: {
        english: 'Logic Design',
        hindi: 'तर्क डिज़ाइन',
        definition: 'The process of designing digital circuits using Boolean algebra and logic gates'
      },
      combinational_circuit: {
        english: 'Combinational Circuit',
        hindi: 'संयोजी परिपथ',
        definition: 'A digital circuit where output depends only on present input values'
      },
      sequential_circuit: {
        english: 'Sequential Circuit',
        hindi: 'अनुक्रमिक परिपथ',
        definition: 'A digital circuit where output depends on both present input and previous states'
      },
      state_machine: {
        english: 'State Machine',
        hindi: 'स्टेट मशीन',
        definition: 'A mathematical model of computation with states and transitions'
      }
    };
  }

  /**
   * Get grading system translation
   */
  getGradingSystem(): Record<string, { english: string; hindi: string }> {
    return {
      outstanding: { english: 'Outstanding (O)', hindi: 'उत्कृष्ट (O)' },
      excellent: { english: 'Excellent (A+)', hindi: 'उत्तम (A+)' },
      very_good: { english: 'Very Good (A)', hindi: 'बहुत अच्छा (A)' },
      good: { english: 'Good (B+)', hindi: 'अच्छा (B+)' },
      above_average: { english: 'Above Average (B)', hindi: 'औसत से ऊपर (B)' },
      average: { english: 'Average (C)', hindi: 'औसत (C)' },
      fail: { english: 'Fail (F)', hindi: 'अनुत्तीर्ण (F)' }
    };
  }

  // Private helper methods
  private detectUserLanguage(): void {
    // Check localStorage first
    const saved = localStorage.getItem('sis_language_preference');
    if (saved && (saved === 'en-IN' || saved === 'hi-IN')) {
      this.currentLanguage = saved as 'en-IN' | 'hi-IN';
      return;
    }

    // Check browser language
    const browserLang = navigator.language;
    if (browserLang === 'hi' || browserLang === 'hi-IN') {
      this.currentLanguage = 'hi-IN';
    } else {
      this.currentLanguage = 'en-IN';
    }
  }

  private getRelativeDate(date: Date): string {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (this.currentLanguage === 'hi-IN') {
      if (diffDays === 0) return 'आज';
      if (diffDays === 1) return 'कल';
      if (diffDays < 7) return `${diffDays} दिन पहले`;
      if (diffDays < 30) return `${Math.floor(diffDays / 7)} सप्ताह पहले`;
      return `${Math.floor(diffDays / 30)} महीने पहले`;
    }

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    return `${Math.floor(diffDays / 30)} months ago`;
  }
}

export default HindiLocalizationService;