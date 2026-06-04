// Consent for optional, privacy-friendly analytics.
//
// The site sets NO analytics or non-essential cookies today, so `ANALYTICS_ENABLED`
// stays false and the banner never appears. We don't ask for consent to nothing.
// When analytics ships: flip this to true and gate the analytics init on
// `consent.analytics === 'granted'`. The visitor's choice persists in localStorage.

export const ANALYTICS_ENABLED = false;

const KEY = 'aleph.consent.analytics';

type Choice = 'granted' | 'denied' | 'unset';

function load(): Choice {
	if (typeof localStorage === 'undefined') return 'unset';
	const stored = localStorage.getItem(KEY);
	return stored === 'granted' || stored === 'denied' ? stored : 'unset';
}

// App-wide reactive state. Mutate `consent.analytics`; never reassign the binding.
export const consent = $state<{ analytics: Choice }>({ analytics: load() });

function choose(value: Choice) {
	consent.analytics = value;
	if (typeof localStorage === 'undefined') return;
	if (value === 'unset') localStorage.removeItem(KEY);
	else localStorage.setItem(KEY, value);
}

export function allowAnalytics() {
	choose('granted');
}

export function declineAnalytics() {
	choose('denied');
}
