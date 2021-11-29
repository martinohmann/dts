use crate::ast::Body;
use crate::ast::{CollectionValue::*, ExprTerm::*, Expression::*, Structure::*};
use crate::parser::{parse, HclParser, Rule};
use pest::*;
use pretty_assertions::assert_eq;

#[test]
fn test_parse() {
    let fixture = std::fs::read_to_string("fixtures/test.tf").unwrap();

    // We could just unwrap here, but we want to pretty print the parse error to make it easier to
    // inspect.
    let body = match parse(&fixture) {
        Ok(body) => body,
        Err(err) => panic!("{}", err),
    };

    let expected: Body = vec![
        Block(
            vec![
                "resource",
                "aws_eks_cluster",
                "this",
            ],
            Box::new(vec![
                Attribute(
                    "count",
                    Conditional(
                        "var.create_eks ? 1 : 0",
                    ),
                ),
                Attribute(
                    "name",
                    ExprTerm(
                        RawExpr(
                            "var.cluster_name",
                        ),
                    ),
                ),
                Attribute(
                    "enabled_cluster_log_types",
                    ExprTerm(
                        RawExpr(
                            "var.cluster_enabled_log_types",
                        ),
                    ),
                ),
                Attribute(
                    "role_arn",
                    ExprTerm(
                        RawExpr(
                            "local.cluster_iam_role_arn",
                        ),
                    ),
                ),
                Attribute(
                    "version",
                    ExprTerm(
                        RawExpr(
                            "var.cluster_version",
                        ),
                    ),
                ),
                Block(
                    vec![
                        "vpc_config",
                    ],
                    Box::new(vec![
                        Attribute(
                            "security_group_ids",
                            ExprTerm(
                                RawExpr(
                                    "compact([local.cluster_security_group_id])",
                                ),
                            ),
                        ),
                        Attribute(
                            "subnet_ids",
                            ExprTerm(
                                RawExpr(
                                    "var.subnets",
                                ),
                            ),
                        ),
                        Attribute(
                            "endpoint_private_access",
                            ExprTerm(
                                RawExpr(
                                    "var.cluster_endpoint_private_access",
                                ),
                            ),
                        ),
                        Attribute(
                            "endpoint_public_access",
                            ExprTerm(
                                RawExpr(
                                    "var.cluster_endpoint_public_access",
                                ),
                            ),
                        ),
                        Attribute(
                            "public_access_cidrs",
                            ExprTerm(
                                RawExpr(
                                    "var.cluster_endpoint_public_access_cidrs",
                                ),
                            ),
                        ),
                    ]),
                ),
                Block(
                    vec![
                        "kubernetes_network_config",
                    ],
                    Box::new(vec![
                        Attribute(
                            "service_ipv4_cidr",
                            ExprTerm(
                                RawExpr(
                                    "var.cluster_service_ipv4_cidr",
                                ),
                            ),
                        ),
                    ]),
                ),
                Block(
                    vec![
                        "dynamic",
                        "encryption_config",
                    ],
                    Box::new(vec![
                        Attribute(
                            "for_each",
                            ExprTerm(
                                RawExpr(
                                    "toset(var.cluster_encryption_config)",
                                ),
                            ),
                        ),
                        Block(
                            vec![
                                "content",
                            ],
                            Box::new(vec![
                                Block(
                                    vec![
                                        "provider",
                                    ],
                                    Box::new(vec![
                                        Attribute(
                                            "key_arn",
                                            ExprTerm(
                                                RawExpr(
                                                    "encryption_config.value[\"provider_key_arn\"]",
                                                ),
                                            ),
                                        ),
                                    ]),
                                ),
                                Attribute(
                                    "resources",
                                    ExprTerm(
                                        RawExpr(
                                            "encryption_config.value[\"resources\"]",
                                        ),
                                    ),
                                ),
                            ]),
                        ),
                    ]),
                ),
                Attribute(
                    "tags",
                    ExprTerm(
                        RawExpr(
                            "merge(\n    var.tags,\n    var.cluster_tags,\n  )",
                        ),
                    ),
                ),
                Block(
                    vec![
                        "timeouts",
                    ],
                    Box::new(vec![
                        Attribute(
                            "create",
                            ExprTerm(
                                RawExpr(
                                    "var.cluster_create_timeout",
                                ),
                            ),
                        ),
                        Attribute(
                            "delete",
                            ExprTerm(
                                RawExpr(
                                    "var.cluster_delete_timeout",
                                ),
                            ),
                        ),
                        Attribute(
                            "update",
                            ExprTerm(
                                RawExpr(
                                    "var.cluster_update_timeout",
                                ),
                            ),
                        ),
                    ]),
                ),
                Attribute(
                    "depends_on",
                    ExprTerm(
                        CollectionValue(
                            Tuple(
                                vec![
                                    ExprTerm(
                                        RawExpr(
                                            "aws_security_group_rule.cluster_egress_internet",
                                        ),
                                    ),
                                    ExprTerm(
                                        RawExpr(
                                            "aws_security_group_rule.cluster_https_worker_ingress",
                                        ),
                                    ),
                                    ExprTerm(
                                        RawExpr(
                                            "aws_iam_role_policy_attachment.cluster_AmazonEKSClusterPolicy",
                                        ),
                                    ),
                                    ExprTerm(
                                        RawExpr(
                                            "aws_iam_role_policy_attachment.cluster_AmazonEKSServicePolicy",
                                        ),
                                    ),
                                    ExprTerm(
                                        RawExpr(
                                            "aws_iam_role_policy_attachment.cluster_AmazonEKSVPCResourceControllerPolicy",
                                        ),
                                    ),
                                    ExprTerm(
                                        RawExpr(
                                            "aws_cloudwatch_log_group.this",
                                        ),
                                    ),
                                ],
                            ),
                        ),
                    ),
                ),
            ]),
        ),
    ];

    assert_eq!(body, expected)
}

#[test]
fn identifier() {
    parses_to! {
        parser: HclParser,
        input: "_an-id3nt1fieR",
        rule: Rule::identifier,
        tokens: [
            identifier(0, 14)
        ]
    };
}

#[test]
fn string() {
    parses_to! {
        parser: HclParser,
        input: "\"a string\"",
        rule: Rule::string_lit,
        tokens: [
            string(1, 9)
        ]
    };
}

#[test]
fn number() {
    parses_to! {
        parser: HclParser,
        input: "-12e+10",
        rule: Rule::numeric_lit,
        tokens: [
            numeric_lit(0, 7, [
                float(0, 7)
            ])
        ]
    };

    parses_to! {
        parser: HclParser,
        input: "42",
        rule: Rule::numeric_lit,
        tokens: [
            numeric_lit(0, 2, [
                int(0, 2)
            ])
        ]
    };
}

#[test]
fn attr() {
    parses_to! {
        parser: HclParser,
        input: "foo = \"bar\"",
        rule: Rule::attribute,
        tokens: [
            attribute(0, 11, [
                identifier(0, 3),
                expression(6, 11, [
                    expr_term(6, 11, [
                        literal_value(6, 11, [
                            string(7, 10)
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn conditional() {
    parses_to! {
        parser: HclParser,
        input: "var.enabled ? 1 : 0",
        rule: Rule::conditional,
        tokens: [
            conditional(0, 19, [
                cond_expr(0, 11, [
                    expr_term(0, 11, [
                        variable_expr(0, 11)
                    ])
                ]),
                expression(14, 15, [
                    expr_term(14, 15, [
                        literal_value(14, 15, [
                            numeric_lit(14, 15, [
                                int(14, 15)
                            ])
                        ])
                    ])
                ]),
                expression(18, 19, [
                    expr_term(18, 19, [
                        literal_value(18, 19, [
                            numeric_lit(18, 19, [
                                int(18, 19)
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn terraform() {
    parses_to! {
        parser: HclParser,
        input: r#"
resource "aws_s3_bucket" "mybucket" {
  bucket        = "mybucket"
  force_destroy = true

  server_side_encryption_configuration {
    rule {
      apply_server_side_encryption_by_default {
        kms_master_key_id = aws_kms_key.mykey.arn
        sse_algorithm     = "aws:kms"
      }
    }
  }
}
            "#,
        rule: Rule::config_file,
        tokens: [
            block(1, 299, [
                block_identifier(1, 36, [
                    identifier(1, 9),
                    string(11, 24),
                    string(27, 35)
                ]),
                block_body(41, 297, [
                    attribute(41, 67, [
                        identifier(41, 47),
                        expression(57, 67, [
                            expr_term(57, 67, [
                                literal_value(57, 67, [
                                    string(58, 66)
                                ])
                            ])
                        ])
                    ]),
                    attribute(70, 90, [
                        identifier(70, 83),
                        expression(86, 90, [
                            expr_term(86, 90, [
                                literal_value(86, 90, [
                                    boolean_lit(86, 90)
                                ])
                            ])
                        ])
                    ]),
                    block(94, 297, [
                        block_identifier(94, 131, [
                            identifier(94, 130)
                        ]),
                        block_body(137, 293, [
                            block(137, 293, [
                                block_identifier(137, 142, [
                                    identifier(137, 141)
                                ]),
                                block_body(150, 287, [
                                    block(150, 287, [
                                        block_identifier(150, 190, [
                                            identifier(150, 189)
                                        ]),
                                        block_body(200, 279, [
                                            attribute(200, 241, [
                                                identifier(200, 217),
                                                expression(220, 241, [
                                                    expr_term(220, 241, [
                                                        variable_expr(220, 241)
                                                    ])
                                                ]),
                                            ]),
                                            attribute(250, 279, [
                                                identifier(250, 263),
                                                expression(270, 279, [
                                                    expr_term(270, 279, [
                                                        literal_value(270, 279, [
                                                            string(271, 278)
                                                        ])
                                                    ])
                                                ])
                                            ])
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn collections() {
    parses_to! {
        parser: HclParser,
        input: r#"foo = ["bar", ["baz"]]"#,
        rule: Rule::attribute,
        tokens: [
            attribute(0, 22, [
                identifier(0, 3),
                expression(6, 22, [
                    expr_term(6, 22, [
                        collection_value(6, 22, [
                            tuple(6, 22, [
                                expression(7, 12, [
                                    expr_term(7, 12, [
                                        literal_value(7, 12, [
                                            string(8, 11)
                                        ])
                                    ])
                                ]),
                                expression(14, 21, [
                                    expr_term(14, 21, [
                                        collection_value(14, 21, [
                                            tuple(14, 21, [
                                                expression(15, 20, [
                                                    expr_term(15, 20, [
                                                        literal_value(15, 20, [
                                                            string(16, 19)
                                                        ])
                                                    ])
                                                ])
                                            ])
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };

    parses_to! {
        parser: HclParser,
        input: r#"foo = {"bar" = "baz","qux" = ident }"#,
        rule: Rule::attribute,
        tokens: [
            attribute(0, 36, [
                identifier(0, 3),
                expression(6, 36, [
                    expr_term(6, 36, [
                        collection_value(6, 36, [
                            object(6, 36, [
                                object_item(7, 20, [
                                    expression(7, 12, [
                                        expr_term(7, 12, [
                                            literal_value(7, 12, [
                                                string(8, 11)
                                            ])
                                        ])
                                    ]),
                                    expression(15, 20, [
                                        expr_term(15, 20, [
                                            literal_value(15, 20, [
                                                string(16, 19)
                                            ])
                                        ])
                                    ]),
                                ]),
                                object_item(21, 34, [
                                    expression(21, 26, [
                                        expr_term(21, 26, [
                                            literal_value(21, 26, [
                                                string(22, 25)
                                            ])
                                        ])
                                    ]),
                                    expression(29, 34, [
                                        expr_term(29, 34, [
                                            variable_expr(29, 34)
                                        ])
                                    ]),
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}
