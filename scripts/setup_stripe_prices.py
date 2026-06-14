#!/usr/bin/env python3
"""
Stripe Price Setup Script for Hardonia Compliance Agent

Creates live Stripe products and prices for the Compliance Agent.
Outputs the price IDs to add to environment variables.

Usage:
  export STRIPE_SECRET_KEY=sk_live_...  python3 scripts/setup_stripe_prices.py
"""

import os
import sys
import json
import subprocess

def run_stripe_cli(args, secret_key=None):
    env = os.environ.copy()
    if secret_key:
        env['STRIPE_SECRET_KEY'] = secret_key
    result = subprocess.run(['stripe'] + args, capture_output=True, text=True, env=env)
    if result.returncode != 0:
        print(f"ERROR: stripe {' '.join(args)}")
        print(result.stderr)
        return None
    return result.stdout

def create_product(name, description, metadata=None):
    args = ['products', 'create', '--name', name, '--description', description]
    if metadata:
        for k, v in metadata.items():
            args.extend(['--metadata[{}]'.format(k), v])
    output = run_stripe_cli(args)
    if output:
        try:
            return json.loads(output.strip().split('\n')[-1])
        except:
            pass
    return None

def create_price(product_id, amount_cents, currency, interval, nickname, metadata=None):
    args = [
        'prices', 'create',
        '--product', product_id,
        '--unit-amount', str(amount_cents),
        '--currency', currency,
        '--recurring[interval]', interval,
        '--nickname', nickname,
    ]
    if metadata:
        for k, v in metadata.items():
            args.extend(['--metadata[{}]'.format(k), v])
    output = run_stripe_cli(args)
    if output:
        try:
            return json.loads(output.strip().split('\n')[-1])
        except:
            pass
    return None

def main():
    secret_key = os.getenv('STRIPE_SECRET_KEY')
    if not secret_key:
        print("ERROR: STRIPE_SECRET_KEY not set")
        sys.exit(1)
    
    if not secret_key.startswith('sk_live_'):
        print("WARNING: Using test key. For production, use sk_live_.")
    
    print("=" * 60)
    print("Compliance Agent Stripe Price Setup")
    print("=" * 60)
    
    metadata = {'product': 'compliance-agent'}
    
    # Starter
    print("\n[1/6] Creating Starter product ($499/mo)...")
    p = create_product("Compliance Agent Starter", "100 tasks/mo, 5 users, email support", metadata)
    if not p: sys.exit(1)
    pid = p['id']
    print(f"  ✓ Product: {pid}")
    
    print("[2/6] Creating Starter price...")
    pr = create_price(pid, 49900, 'usd', 'month', 'Starter Monthly', metadata)
    if not pr: sys.exit(1)
    print(f"  ✓ Price: {pr['id']}")
    starter_price = pr['id']
    
    # Growth
    print("\n[3/6] Creating Growth product ($1,499/mo)...")
    p = create_product("Compliance Agent Growth", "500 tasks/mo, 20 users, priority support", metadata)
    if not p: sys.exit(1)
    pid = p['id']
    print(f"  ✓ Product: {pid}")
    
    print("[4/6] Creating Growth price...")
    pr = create_price(pid, 149900, 'usd', 'month', 'Growth Monthly', metadata)
    if not pr: sys.exit(1)
    print(f"  ✓ Price: {pr['id']}")
    growth_price = pr['id']
    
    # Enterprise
    print("\n[5/6] Creating Enterprise product ($4,999/mo)...")
    p = create_product("Compliance Agent Enterprise", "Unlimited tasks, unlimited users, dedicated support", metadata)
    if not p: sys.exit(1)
    pid = p['id']
    print(f"  ✓ Product: {pid}")
    
    print("[6/6] Creating Enterprise price...")
    pr = create_price(pid, 499900, 'usd', 'month', 'Enterprise Monthly', metadata)
    if not pr: sys.exit(1)
    print(f"  ✓ Price: {pr['id']}")
    enterprise_price = pr['id']
    
    print("\n" + "=" * 60)
    print("SETUP COMPLETE")
    print("=" * 60)
    print(f"""
# Add to .env
STRIPE_PRICE_STARTER={starter_price}
STRIPE_PRICE_GROWTH={growth_price}
STRIPE_PRICE_ENTERPRISE={enterprise_price}

# Webhook URL: https://api.compliance.hardonia.com/api/v1/stripe/webhook
# Webhook events: checkout.session.completed, customer.subscription.*, invoice.*
""")

if __name__ == '__main__':
    main()