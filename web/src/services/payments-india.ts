/**
 * Indian Payment Service Integration
 * Phase 5A: UPI, Razorpay, GST Compliance, and Indian Banking
 */

import { INDIAN_MARKET_CONFIG } from '../config/infrastructure';

export interface IndianPaymentMethod {
  id: string;
  name: string;
  type: 'upi' | 'netbanking' | 'card' | 'wallet' | 'emi';
  provider: string;
  fees: number;
  processingTime: string;
  availability: 'always' | 'business_hours' | 'weekdays';
}

export interface GSTDetails {
  gstin: string;
  businessName: string;
  address: {
    street: string;
    city: string;
    state: string;
    pincode: string;
  };
  gstRate: number;
  hsnCode: string; // Harmonized System of Nomenclature
}

export interface IndianTransaction {
  id: string;
  userId: string;
  amount: number;
  currency: 'INR';
  gstAmount: number;
  tdsAmount: number;
  finalAmount: number;
  paymentMethod: IndianPaymentMethod;
  gstDetails: GSTDetails;
  razorpayOrderId?: string;
  upiTransactionId?: string;
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'refunded';
  createdAt: Date;
  completedAt?: Date;
}

export class IndianPaymentService {
  private razorpayApiKey: string;
  private razorpaySecret: string;
  private gstNumber: string;

  constructor() {
    this.razorpayApiKey = process.env.RAZORPAY_KEY_ID!;
    this.razorpaySecret = process.env.RAZORPAY_KEY_SECRET!;
    this.gstNumber = process.env.COMPANY_GST_NUMBER!;
  }

  /**
   * Calculate GST and final amount for Indian transactions
   */
  calculateIndianTaxes(baseAmount: number, userGST?: string): {
    baseAmount: number;
    gstAmount: number;
    tdsAmount: number;
    finalAmount: number;
    breakdown: {
      cgst: number;
      sgst: number;
      igst: number;
    };
  } {
    const { gst, tds } = INDIAN_MARKET_CONFIG.pricing.taxation;
    
    // GST calculation
    const gstAmount = baseAmount * gst.rate;
    
    // TDS calculation (if applicable - usually for B2B transactions above threshold)
    const tdsAmount = baseAmount > tds.threshold ? baseAmount * tds.rate : 0;
    
    // GST breakdown (simplified - would need user's state for accurate CGST/SGST)
    const isInterState = this.isInterStateTransaction(userGST);
    const gstBreakdown = isInterState 
      ? { cgst: 0, sgst: 0, igst: gstAmount }  // Inter-state: IGST
      : { cgst: gstAmount / 2, sgst: gstAmount / 2, igst: 0 }; // Intra-state: CGST + SGST

    const finalAmount = baseAmount + gstAmount - tdsAmount;

    return {
      baseAmount,
      gstAmount,
      tdsAmount,
      finalAmount,
      breakdown: gstBreakdown
    };
  }

  /**
   * Create Razorpay order for Indian payment
   */
  async createRazorpayOrder(
    amount: number, 
    _currency: 'INR',
    userDetails: {
      email: string;
      phone: string;
      name: string;
    },
    subscriptionTier: string
  ): Promise<{
    orderId: string;
    amount: number;
    currency: string;
    receipt: string;
  }> {
    const taxes = this.calculateIndianTaxes(amount);
    
    const orderData = {
      amount: Math.round(taxes.finalAmount * 100), // Razorpay expects amount in paise
      currency: 'INR',
      receipt: `sis_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      notes: {
        subscription_tier: subscriptionTier,
        base_amount: taxes.baseAmount,
        gst_amount: taxes.gstAmount,
        tds_amount: taxes.tdsAmount,
        user_email: userDetails.email,
        company_gstn: this.gstNumber
      }
    };

    try {
      // Mock Razorpay API call (replace with actual Razorpay SDK)
      const response = await fetch('https://api.razorpay.com/v1/orders', {
        method: 'POST',
        headers: {
          'Authorization': `Basic ${Buffer.from(`${this.razorpayApiKey}:${this.razorpaySecret}`).toString('base64')}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(orderData),
      });

      const order = await response.json();
      
      return {
        orderId: order.id,
        amount: order.amount,
        currency: order.currency,
        receipt: order.receipt
      };
    } catch (error) {
      console.error('Razorpay order creation failed:', error);
      throw new Error('Failed to create payment order');
    }
  }

  /**
   * Process UPI payment
   */
  async processUPIPayment(
    vpa: string, // Virtual Payment Address like user@paytm
    amount: number,
    description: string
  ): Promise<{
    transactionId: string;
    status: 'success' | 'pending' | 'failed';
    upiRef: string;
  }> {
    // UPI payment processing through Razorpay
    const upiPayment = {
      vpa,
      amount: Math.round(amount * 100), // in paise
      description,
      currency: 'INR'
    };

    try {
      // Mock UPI processing (replace with actual Razorpay UPI API)
      const response = await this.mockUPIProcess(upiPayment);
      
      return {
        transactionId: response.transactionId,
        status: response.status,
        upiRef: response.upiRef
      };
    } catch (error) {
      console.error('UPI payment processing failed:', error);
      throw new Error('UPI payment failed');
    }
  }

  /**
   * Generate GST-compliant invoice
   */
  async generateGSTInvoice(transaction: IndianTransaction): Promise<{
    invoiceNumber: string;
    invoiceDate: string;
    gstDetails: GSTDetails;
    itemDetails: {
      description: string;
      hsnCode: string;
      quantity: number;
      rate: number;
      taxableValue: number;
      gstRate: number;
      gstAmount: number;
      totalAmount: number;
    };
    totalInWords: string;
  }> {
    const invoiceNumber = `SIS/INV/${new Date().getFullYear()}/${String(Date.now()).slice(-6)}`;
    const invoiceDate = new Date().toISOString().split('T')[0];
    
    // HSN Code for Software/Digital Services in India
    const hsnCode = '998314'; // Computer software (customised)
    
    const itemDetails = {
      description: 'SIS AI-Lab Platform Subscription',
      hsnCode,
      quantity: 1,
      rate: transaction.amount - transaction.gstAmount,
      taxableValue: transaction.amount - transaction.gstAmount,
      gstRate: INDIAN_MARKET_CONFIG.pricing.taxation.gst.rate * 100,
      gstAmount: transaction.gstAmount,
      totalAmount: transaction.finalAmount
    };

    const totalInWords = this.convertAmountToWords(transaction.finalAmount);

    return {
      invoiceNumber,
      invoiceDate,
      gstDetails: transaction.gstDetails,
      itemDetails,
      totalInWords
    };
  }

  /**
   * EMI (Equated Monthly Installments) calculation
   */
  calculateEMI(
    amount: number,
    durationMonths: number,
    interestRate: number = 12 // 12% annual interest rate
  ): {
    monthlyEMI: number;
    totalAmount: number;
    totalInterest: number;
    emiBreakdown: Array<{
      month: number;
      emiAmount: number;
      principalAmount: number;
      interestAmount: number;
      remainingBalance: number;
    }>;
  } {
    const monthlyRate = interestRate / (12 * 100);
    const monthlyEMI = (amount * monthlyRate * Math.pow(1 + monthlyRate, durationMonths)) /
                       (Math.pow(1 + monthlyRate, durationMonths) - 1);
    
    const totalAmount = monthlyEMI * durationMonths;
    const totalInterest = totalAmount - amount;

    // Generate EMI breakdown
    const emiBreakdown = [];
    let remainingBalance = amount;

    for (let month = 1; month <= durationMonths; month++) {
      const interestAmount = remainingBalance * monthlyRate;
      const principalAmount = monthlyEMI - interestAmount;
      remainingBalance -= principalAmount;

      emiBreakdown.push({
        month,
        emiAmount: Math.round(monthlyEMI * 100) / 100,
        principalAmount: Math.round(principalAmount * 100) / 100,
        interestAmount: Math.round(interestAmount * 100) / 100,
        remainingBalance: Math.max(0, Math.round(remainingBalance * 100) / 100)
      });
    }

    return {
      monthlyEMI: Math.round(monthlyEMI * 100) / 100,
      totalAmount: Math.round(totalAmount * 100) / 100,
      totalInterest: Math.round(totalInterest * 100) / 100,
      emiBreakdown
    };
  }

  /**
   * Indian banking holidays and business hours validation
   */
  isIndianBankingHour(): boolean {
    const now = new Date();
    const istTime = new Date(now.toLocaleString('en-US', { timeZone: 'Asia/Kolkata' }));
    const hour = istTime.getHours();
    const day = istTime.getDay();

    // Monday to Friday: 10 AM to 6 PM IST
    // Saturday: 10 AM to 2 PM IST
    // Sunday: Closed
    if (day === 0) return false; // Sunday
    if (day === 6) return hour >= 10 && hour < 14; // Saturday
    return hour >= 10 && hour < 18; // Monday to Friday
  }

  /**
   * Regional pricing based on user location in India
   */
  getRegionalPricing(
    basePrice: number,
    userLocation: {
      city: string;
      state: string;
      pincode: string;
    }
  ): {
    originalPrice: number;
    discount: number;
    finalPrice: number;
    discountReason: string;
  } {
    const { regionalDiscounts } = INDIAN_MARKET_CONFIG.pricing;
    
    let discount = 0;
    let discountReason = '';

    // Determine city tier
    const cityLower = userLocation.city.toLowerCase();
    const tier1Cities = ['mumbai', 'delhi', 'bangalore', 'hyderabad', 'chennai', 'pune', 'kolkata', 'ahmedabad'];
    if (tier1Cities.includes(cityLower)) {
      discount = 0;
      discountReason = 'Tier 1 City - Standard Pricing';
    } else if (this.isTier2City(cityLower)) {
      discount = regionalDiscounts.tier2Cities.discount;
      discountReason = 'Tier 2 City - 20% Discount';
    } else if (this.isTier3City(cityLower)) {
      discount = regionalDiscounts.tier3Cities.discount;
      discountReason = 'Tier 3 City - 40% Discount';
    } else {
      discount = regionalDiscounts.ruralAreas.discount;
      discountReason = 'Rural Area - 50% Discount (Government Partnership)';
    }

    const finalPrice = basePrice * (1 - discount);

    return {
      originalPrice: basePrice,
      discount: discount * 100, // percentage
      finalPrice: Math.round(finalPrice),
      discountReason
    };
  }

  // Helper methods
  private isInterStateTransaction(userGST?: string): boolean {
    if (!userGST) return true; // Default to inter-state for B2C
    const userStateCode = userGST.substring(0, 2);
    const companyStateCode = this.gstNumber.substring(0, 2);
    return userStateCode !== companyStateCode;
  }

  private async mockUPIProcess(_payment: any): Promise<any> {
    // Mock UPI processing - replace with actual Razorpay UPI API
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve({
          transactionId: `upi_${Date.now()}`,
          status: 'success',
          upiRef: `UPI${Date.now().toString().slice(-10)}`
        });
      }, 2000);
    });
  }

  private convertAmountToWords(amount: number): string {
    // Simplified amount to words conversion for Indian format
    // In production, use a proper number-to-words library
    const crores = Math.floor(amount / 10000000);
    const lakhs = Math.floor((amount % 10000000) / 100000);
    const thousands = Math.floor((amount % 100000) / 1000);
    const hundreds = Math.floor((amount % 1000) / 100);
    const remainder = amount % 100;

    let words = '';
    if (crores > 0) words += `${crores} Crore `;
    if (lakhs > 0) words += `${lakhs} Lakh `;
    if (thousands > 0) words += `${thousands} Thousand `;
    if (hundreds > 0) words += `${hundreds} Hundred `;
    if (remainder > 0) words += `${remainder}`;
    
    return `${words} Rupees Only`.trim();
  }

  private isTier2City(city: string): boolean {
    const tier2Cities = ['pune', 'jaipur', 'lucknow', 'kanpur', 'nagpur', 'indore', 'thane', 'bhopal', 'visakhapatnam', 'pimpri', 'patna', 'vadodara', 'ghaziabad', 'ludhiana', 'coimbatore'];
    return tier2Cities.includes(city);
  }

  private isTier3City(city: string): boolean {
    // Simplified - in production, use comprehensive city classification
    const tier1Cities = ['mumbai', 'delhi', 'bangalore', 'hyderabad', 'chennai', 'pune', 'kolkata', 'ahmedabad'];
    return !this.isTier2City(city) && !tier1Cities.includes(city);
  }
}

export default IndianPaymentService;