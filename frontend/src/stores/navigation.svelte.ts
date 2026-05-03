import type { Route } from '@types';

function createNavigationStore() {
  let current = $state<Route>('dashboard');

  return {
    get current() {
      return current;
    },
    navigate(route: Route) {
      current = route;
    }
  };
}

export const navigation = createNavigationStore();
