/** @type {import('tailwindcss').Config} */
module.exports = {
	darkMode: 'class',
	content: ['./src/**/*.{html,js,svelte,ts}', require('path').join(require.resolve('@skeletonlabs/skeleton'), '../**/*.{html,js,svelte,ts}')],
	theme: {
		extend: {
			backgroundImage: {
				'amber-logo':"url('/amber-logo.png')",
			  },
			transitionTimingFunction: {
				'standard': 'cubic-bezier(0.2, 0, 0, 1)',
				'standard-decelerate': 'cubic-bezier(0, 0, 0, 1)',
				'standard-accelerate': 'cubic-bezier(0.3, 0.1, 1, 1)',
				'emphasized-decelerate': 'cubic-bezier(0.05, 0.7, 0.1, 1.0)',
				'emphasized-accelerate': 'cubic-bezier(0.3, 0.0, 0.8, 0.15)',
			},
		},
	},
	plugins: [
		require('@tailwindcss/forms'),
		...require('@skeletonlabs/skeleton/tailwind/skeleton.cjs')()],
}
