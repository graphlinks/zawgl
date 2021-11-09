package io.onegraph.gremlin.structure;

import org.apache.commons.configuration2.BaseConfiguration;
import org.apache.commons.configuration2.Configuration;
import org.apache.tinkerpop.gremlin.driver.Cluster;
import org.apache.tinkerpop.gremlin.driver.remote.DriverRemoteConnection;
import org.apache.tinkerpop.gremlin.driver.ser.Serializers;
import org.apache.tinkerpop.gremlin.process.computer.GraphComputer;
import org.apache.tinkerpop.gremlin.process.traversal.AnonymousTraversalSource;
import org.apache.tinkerpop.gremlin.process.traversal.dsl.graph.GraphTraversalSource;
import org.apache.tinkerpop.gremlin.structure.Edge;
import org.apache.tinkerpop.gremlin.structure.Graph;
import org.apache.tinkerpop.gremlin.structure.Transaction;
import org.apache.tinkerpop.gremlin.structure.Vertex;
import org.apache.tinkerpop.gremlin.structure.util.ElementHelper;
import org.apache.tinkerpop.gremlin.structure.util.StringFactory;

import java.util.Collections;
import java.util.Iterator;
import java.util.Optional;

@Graph.OptIn(Graph.OptIn.SUITE_STRUCTURE_STANDARD)
@Graph.OptIn(Graph.OptIn.SUITE_PROCESS_STANDARD)
@Graph.OptIn(Graph.OptIn.SUITE_PROCESS_COMPUTER)
public class OneGraph implements Graph {

    private static final Configuration EMPTY_CONFIGURATION = new BaseConfiguration() {{
        this.setProperty(Graph.GRAPH, OneGraph.class.getName());
    }};
    private static ThreadLocal<GraphTraversalSource> graphTraversalSourceThreadLocal = ThreadLocal.withInitial(() -> createSource(createCluster()));

    protected final BaseConfiguration configuration = new BaseConfiguration();;

    public OneGraph(Configuration configuration) {
        this.configuration.copy(configuration);
    }

    public static OneGraph open() {
        return OneGraph.open(EMPTY_CONFIGURATION);
    }

    public static OneGraph open(final Configuration configuration) {
        return new OneGraph(Optional.ofNullable(configuration).orElse(EMPTY_CONFIGURATION));
    }

    @Override
    public Vertex addVertex(Object... keyValues) {
        ElementHelper.legalPropertyKeyValueArray(keyValues);
        if (ElementHelper.getIdValue(keyValues).isPresent())
            throw Vertex.Exceptions.userSuppliedIdsNotSupported();
        //graphTraversalSourceThreadLocal.get().addV()
        throw Exceptions.vertexAdditionsNotSupported();
    }

    @Override
    public void close() throws Exception {
        configuration.clear();
        graphTraversalSourceThreadLocal.get().close();
    }

    @Override
    public GraphComputer compute() throws IllegalArgumentException {
        throw Exceptions.graphComputerNotSupported();
    }

    @Override
    public <C extends GraphComputer> C compute(Class<C> graphComputerClass) throws IllegalArgumentException {
        throw Exceptions.graphComputerNotSupported();
    }

    @Override
    public Configuration configuration() {
        return configuration;
    }

    @Override
    public Iterator<Edge> edges(Object... edgeIds) {
        return Collections.emptyIterator();
    }

    @Override
    public Transaction tx() {
        return graphTraversalSourceThreadLocal.get().tx();
    }

    @Override
    public Variables variables() {
        throw Exceptions.variablesNotSupported();
    }

    @Override
    public Iterator<Vertex> vertices(Object... vertexIds) {
        return Collections.emptyIterator();
    }

    @Override
    public GraphTraversalSource traversal() {
        return graphTraversalSourceThreadLocal.get();
    }

    private static Cluster createCluster() {
        final Cluster cluster = Cluster.build("localhost")
                .port(8182)
                .maxInProcessPerConnection(32)
                .maxSimultaneousUsagePerConnection(32)
                .serializer(Serializers.GRAPHSON_V3D0)
                .create();
        return cluster;
    }

    private static GraphTraversalSource createSource(final Cluster cluster) {
        final GraphTraversalSource g = AnonymousTraversalSource.traversal().withRemote(DriverRemoteConnection.using(cluster));
        return g;
    }

    @Override
    public Features features() {
        return new OneGraphFeatures();
    }

    public static class OneGraphFeatures implements Features {

        @Override
        public GraphFeatures graph() {
            return new GraphFeatures() {

                @Override
                public boolean supportsTransactions() {
                    return false;
                }

                @Override
                public boolean supportsThreadedTransactions() {
                    return false;
                }

                @Override
                public Features.VariableFeatures variables() {
                    return new Features.VariableFeatures() {
                        @Override
                        public boolean supportsVariables() {
                            return false;
                        }

                        @Override
                        public boolean supportsBooleanValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsByteValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsDoubleValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsFloatValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsIntegerValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsLongValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsMapValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsMixedListValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsBooleanArrayValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsByteArrayValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsDoubleArrayValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsFloatArrayValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsIntegerArrayValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsStringArrayValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsLongArrayValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsSerializableValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsStringValues() {
                            return false;
                        }

                        @Override
                        public boolean supportsUniformListValues() {
                            return false;
                        }
                    };
                }
            };
        }

        @Override
        public EdgeFeatures edge() {
            return new EdgeFeatures() {
                @Override
                public boolean supportsAddEdges() {
                    return false;
                }

                @Override
                public boolean supportsRemoveEdges() {
                    return false;
                }

                @Override
                public boolean supportsAddProperty() {
                    return false;
                }

                @Override
                public boolean supportsRemoveProperty() {
                    return false;
                }

                @Override
                public boolean supportsCustomIds() {
                    return false;
                }
            };
        }

        @Override
        public VertexFeatures vertex() {
            return new VertexFeatures() {
                @Override
                public boolean supportsAddVertices() {
                    return false;
                }

                @Override
                public boolean supportsRemoveVertices() {
                    return false;
                }

                @Override
                public boolean supportsAddProperty() {
                    return false;
                }

                @Override
                public boolean supportsRemoveProperty() {
                    return false;
                }

                @Override
                public boolean supportsCustomIds() {
                    return false;
                }

                @Override
                public Features.VertexPropertyFeatures properties() {
                    return new Features.VertexPropertyFeatures() {
                        @Override
                        public boolean supportsRemoveProperty() {
                            return false;
                        }

                        @Override
                        public boolean supportsCustomIds() {
                            return false;
                        }
                    };
                }
            };
        }

        @Override
        public String toString() {
            return StringFactory.featureString(this);
        }
    }


}
