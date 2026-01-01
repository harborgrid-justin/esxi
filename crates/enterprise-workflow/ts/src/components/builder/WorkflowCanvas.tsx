/**
 * Workflow Canvas - Drag-and-drop workflow builder canvas
 */

import React, { useCallback, useState } from 'react';
import ReactFlow, {
  Node,
  Edge,
  Controls,
  Background,
  MiniMap,
  addEdge,
  Connection,
  useNodesState,
  useEdgesState,
  ReactFlowProvider,
  MarkerType
} from 'reactflow';
import 'reactflow/dist/style.css';
import { Workflow, WorkflowStep, Transition } from '../../types';

export interface WorkflowCanvasProps {
  workflow?: Workflow;
  onWorkflowChange?: (workflow: Workflow) => void;
  readonly?: boolean;
}

const nodeTypes = {
  action: ActionNode,
  condition: ConditionNode,
  parallel: ParallelNode,
  loop: LoopNode,
  wait: WaitNode,
  subworkflow: SubworkflowNode
};

export const WorkflowCanvas: React.FC<WorkflowCanvasProps> = ({
  workflow,
  onWorkflowChange,
  readonly = false
}) => {
  const [nodes, setNodes, onNodesChange] = useNodesState(
    workflow ? convertStepsToNodes(workflow.steps) : []
  );
  const [edges, setEdges, onEdgesChange] = useEdgesState(
    workflow ? convertTransitionsToEdges(workflow.steps) : []
  );
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);

  /**
   * Handle new connections between nodes
   */
  const onConnect = useCallback(
    (connection: Connection) => {
      if (readonly) return;

      const edge = {
        ...connection,
        type: 'smoothstep',
        animated: true,
        markerEnd: {
          type: MarkerType.ArrowClosed
        }
      };

      setEdges((eds) => addEdge(edge, eds));

      // Update workflow
      if (workflow && onWorkflowChange) {
        const updatedWorkflow = addTransitionToWorkflow(workflow, connection);
        onWorkflowChange(updatedWorkflow);
      }
    },
    [workflow, onWorkflowChange, readonly, setEdges]
  );

  /**
   * Handle node selection
   */
  const onNodeClick = useCallback((event: React.MouseEvent, node: Node) => {
    setSelectedNode(node);
  }, []);

  /**
   * Handle node deletion
   */
  const onNodesDelete = useCallback(
    (nodesToDelete: Node[]) => {
      if (readonly) return;

      if (workflow && onWorkflowChange) {
        const updatedSteps = workflow.steps.filter(
          step => !nodesToDelete.find(n => n.id === step.id)
        );

        onWorkflowChange({
          ...workflow,
          steps: updatedSteps
        });
      }
    },
    [workflow, onWorkflowChange, readonly]
  );

  /**
   * Handle edge deletion
   */
  const onEdgesDelete = useCallback(
    (edgesToDelete: Edge[]) => {
      if (readonly) return;

      if (workflow && onWorkflowChange) {
        const updatedWorkflow = removeTransitionsFromWorkflow(workflow, edgesToDelete);
        onWorkflowChange(updatedWorkflow);
      }
    },
    [workflow, onWorkflowChange, readonly]
  );

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
        onNodesDelete={onNodesDelete}
        onEdgesDelete={onEdgesDelete}
        nodeTypes={nodeTypes}
        fitView
        deleteKeyCode={readonly ? null : 'Delete'}
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
};

/**
 * Action Node Component
 */
function ActionNode({ data }: any) {
  return (
    <div
      style={{
        padding: '12px 20px',
        borderRadius: '8px',
        background: '#4CAF50',
        color: 'white',
        border: '2px solid #388E3C',
        minWidth: '150px',
        textAlign: 'center'
      }}
    >
      <div style={{ fontWeight: 'bold' }}>{data.label}</div>
      {data.actionType && (
        <div style={{ fontSize: '12px', opacity: 0.9 }}>
          {data.actionType}
        </div>
      )}
    </div>
  );
}

/**
 * Condition Node Component
 */
function ConditionNode({ data }: any) {
  return (
    <div
      style={{
        padding: '12px 20px',
        borderRadius: '8px',
        background: '#FF9800',
        color: 'white',
        border: '2px solid #F57C00',
        minWidth: '150px',
        textAlign: 'center',
        clipPath: 'polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)'
      }}
    >
      <div style={{ fontWeight: 'bold' }}>{data.label}</div>
    </div>
  );
}

/**
 * Parallel Node Component
 */
function ParallelNode({ data }: any) {
  return (
    <div
      style={{
        padding: '12px 20px',
        borderRadius: '8px',
        background: '#2196F3',
        color: 'white',
        border: '2px solid #1976D2',
        minWidth: '150px',
        textAlign: 'center'
      }}
    >
      <div style={{ fontWeight: 'bold' }}>{data.label}</div>
      <div style={{ fontSize: '12px', opacity: 0.9 }}>
        {data.branches} branches
      </div>
    </div>
  );
}

/**
 * Loop Node Component
 */
function LoopNode({ data }: any) {
  return (
    <div
      style={{
        padding: '12px 20px',
        borderRadius: '50%',
        background: '#9C27B0',
        color: 'white',
        border: '2px solid #7B1FA2',
        minWidth: '150px',
        minHeight: '150px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        textAlign: 'center'
      }}
    >
      <div>
        <div style={{ fontWeight: 'bold' }}>{data.label}</div>
        <div style={{ fontSize: '12px', opacity: 0.9 }}>
          {data.loopType}
        </div>
      </div>
    </div>
  );
}

/**
 * Wait Node Component
 */
function WaitNode({ data }: any) {
  return (
    <div
      style={{
        padding: '12px 20px',
        borderRadius: '8px',
        background: '#607D8B',
        color: 'white',
        border: '2px solid #455A64',
        minWidth: '150px',
        textAlign: 'center'
      }}
    >
      <div style={{ fontWeight: 'bold' }}>{data.label}</div>
      <div style={{ fontSize: '12px', opacity: 0.9 }}>
        Wait: {data.waitType}
      </div>
    </div>
  );
}

/**
 * Subworkflow Node Component
 */
function SubworkflowNode({ data }: any) {
  return (
    <div
      style={{
        padding: '12px 20px',
        borderRadius: '8px',
        background: '#00BCD4',
        color: 'white',
        border: '2px solid #0097A7',
        minWidth: '150px',
        textAlign: 'center',
        boxShadow: '0 4px 8px rgba(0,0,0,0.2)'
      }}
    >
      <div style={{ fontWeight: 'bold' }}>{data.label}</div>
      <div style={{ fontSize: '12px', opacity: 0.9 }}>
        Subworkflow
      </div>
    </div>
  );
}

/**
 * Convert workflow steps to React Flow nodes
 */
function convertStepsToNodes(steps: WorkflowStep[]): Node[] {
  return steps.map(step => ({
    id: step.id,
    type: step.type,
    position: step.position,
    data: {
      label: step.name,
      actionType: step.action?.type,
      branches: step.parallelBranches?.length,
      loopType: step.loopConfig?.type,
      waitType: step.waitConfig?.type
    }
  }));
}

/**
 * Convert workflow transitions to React Flow edges
 */
function convertTransitionsToEdges(steps: WorkflowStep[]): Edge[] {
  const edges: Edge[] = [];

  steps.forEach(step => {
    step.transitions.forEach((transition, index) => {
      edges.push({
        id: `${step.id}-${transition.to}-${index}`,
        source: step.id,
        target: transition.to,
        type: 'smoothstep',
        animated: true,
        label: transition.label,
        markerEnd: {
          type: MarkerType.ArrowClosed
        }
      });
    });
  });

  return edges;
}

/**
 * Add transition to workflow
 */
function addTransitionToWorkflow(workflow: Workflow, connection: Connection): Workflow {
  const steps = [...workflow.steps];
  const sourceStep = steps.find(s => s.id === connection.source);

  if (sourceStep && connection.target) {
    const transition: Transition = {
      id: `${connection.source}-${connection.target}`,
      from: connection.source,
      to: connection.target
    };

    sourceStep.transitions.push(transition);
  }

  return { ...workflow, steps };
}

/**
 * Remove transitions from workflow
 */
function removeTransitionsFromWorkflow(workflow: Workflow, edges: Edge[]): Workflow {
  const steps = workflow.steps.map(step => ({
    ...step,
    transitions: step.transitions.filter(
      t => !edges.find(e => e.source === t.from && e.target === t.to)
    )
  }));

  return { ...workflow, steps };
}

// Wrapper with provider
export const WorkflowCanvasWithProvider: React.FC<WorkflowCanvasProps> = (props) => (
  <ReactFlowProvider>
    <WorkflowCanvas {...props} />
  </ReactFlowProvider>
);
