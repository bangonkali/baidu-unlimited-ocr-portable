import type { Meta, StoryObj } from '@storybook/react-vite';

import './storybook.css';

const meta = {
  title: 'Design System/Component Inventory',
} satisfies Meta;

export default meta;

type Story = StoryObj<typeof meta>;

const components = [
  'IngestToolbar',
  'ExplorerTree',
  'PreviewPane',
  'TextPane',
  'DetailsPane',
  'DiagnosticsPanel',
  'StatusBar',
  'IconButton',
];

export const Inventory: Story = {
  render: () => (
    <div className="inventory">
      {components.map((component) => (
        <div className="inventoryRow" key={component}>
          {component}
        </div>
      ))}
    </div>
  ),
};
