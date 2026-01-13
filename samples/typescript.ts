import {
  type EdgeModel,
  type NodeModel,
  type RootGraphModel,
  type SubgraphModel,
  attribute,
  fromDot,
  toDot,
} from "ts-graphviz";
import type { ColorScheme } from "../control-flow/colors.ts";
import {
  type ClusterClass,
  getClusterStyle,
  getEdgeDefaultStyle,
  getEdgeStyle,
  getNodeHeight,
  getNodeStyle,
  isClusterClass,
  isEdgeClass,
  isNodeClass,
} from "./theme.ts";

/**
 * Get the classes for a given edge or node
 * @param item The edge or node
 * @param defaultClass A default class name to use if no classes are found
 */
function getClasses(
  item: NodeModel | EdgeModel,
  defaultClass: string,
): string[] {
  type ClassGetter = (key: (typeof attribute)["class"]) => string | undefined;
  const classGetter: ClassGetter = item.attributes.get.bind(item.attributes);
  const classString = classGetter(attribute.class);
  return (classString ?? defaultClass).split(/\s/);
}

export function applyTheme(dot: string, colorScheme: ColorScheme): string {
  const G = fromDot(dot);
  G.apply({
    [attribute.bgcolor]: colorScheme["graph.background"],
  });
  const nodeAttrs = getNodeStyle("default", colorScheme);
  nodeAttrs.fontname = "sans-serif";
  nodeAttrs.height = getNodeHeight("default", 0);
  G.node(nodeAttrs);
  G.edge(getEdgeDefaultStyle(colorScheme));

  for (const node of iterAllNodes(G)) {
    const originalLabel = node.attributes.get("label");
    for (const cls of getClasses(node, "default")) {
      if (isNodeClass(cls)) {
        node.attributes.apply(getNodeStyle(cls, colorScheme));
        const height = node.attributes.get("height");
        let lines = 0;
        if (typeof height === "number") {
          lines = height;
        } else if (typeof height === "string") {
          lines = Number.parseFloat(height);
        }
        node.attributes.set("height", getNodeHeight(cls, lines));
      }
    }
    if (originalLabel) {
      node.attributes.set("label", originalLabel);
    }
  }

  for (const edge of iterAllEdges(G)) {
    const isBacklink = edge.attributes.get(attribute.dir) === "back";
    if (isBacklink) {
      edge.targets.reverse();
    }

    for (const cls of getClasses(edge, "regular")) {
      if (isEdgeClass(cls)) {
        edge.attributes.apply(getEdgeStyle(cls, isBacklink, colorScheme));
      }
    }
  }

  for (const { subgraph, isSelfNested } of iterAllSubgraphs(G)) {
    const classes = subgraph.get("class");
    if (!classes) {
      continue;
    }

    for (const cls of classes.split(/\s/).filter(isClusterClass)) {
      subgraph.attributes.graph.apply(
        getClusterStyle(cls, isSelfNested, colorScheme),
      );
    }
  }

  return toDot(G);
}

function* iterAllNodes(G: RootGraphModel) {
  const stack: Array<RootGraphModel | SubgraphModel> = [G];
  for (;;) {
    const graph = stack.pop();
    if (!graph) {
      break;
    }
    stack.push(...graph.subgraphs);
    yield* graph.nodes;
  }
}

function* iterAllEdges(G: RootGraphModel) {
  const stack: Array<RootGraphModel | SubgraphModel> = [G];
  for (;;) {
    const graph = stack.pop();
    if (!graph) {
      break;
    }
    stack.push(...graph.subgraphs);
    yield* graph.edges;
  }
}

function* iterAllSubgraphs(G: RootGraphModel) {
  const stack: Array<{
    graph: SubgraphModel;
    parentClass: ClusterClass | undefined;
  }> = G.subgraphs.map((subgraph) => ({
    graph: subgraph,
    parentClass: undefined,
  }));
  for (;;) {
    const entry = stack.pop();
    if (!entry) {
      break;
    }
    const { graph, parentClass } = entry;
    const clusterClass = (graph.get("class") ?? "")
      .split(/\s/)
      .filter(isClusterClass)
      .pop();
    stack.push(
      ...graph.subgraphs.map((subgraph) => ({
        graph: subgraph,
        parentClass: clusterClass,
      })),
    );
    yield { subgraph: graph, isSelfNested: clusterClass === parentClass };
  }
}
