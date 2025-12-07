//! Tests for the Visitable trait implementation

use typhon_ast::ast::AST;
use typhon_ast::nodes::{
    AnyNode,
    BasicIdent,
    BinaryOpExpr,
    BinaryOpKind,
    BreakStmt,
    CallableType,
    ContinueStmt,
    DictExpr,
    FromImportStmt,
    GenericType,
    ImportStmt,
    LambdaExpr,
    ListExpr,
    LiteralExpr,
    LiteralValue,
    Module,
    NodeID,
    NodeKind,
    PassStmt,
    TupleExpr,
    UnaryOpExpr,
    UnaryOpKind,
    UnionType,
};
use typhon_ast::visitor::{Visitable, Visitor, VisitorResult};
use typhon_source::types::Span;

// Create a test visitor to track visited node types
struct TestVisitor {
    visited_nodes: Vec<&'static str>,
}

impl TestVisitor {
    const fn new() -> Self { Self { visited_nodes: Vec::new() } }
}

impl Visitor<()> for TestVisitor {
    fn visit(&mut self, _node_id: NodeID) -> Option<()> {
        self.visited_nodes.push("visit");
        None
    }

    fn visit_binary_op_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_binary_op_expr");
        Ok(())
    }

    fn visit_unary_op_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_unary_op_expr");
        Ok(())
    }

    fn visit_literal_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_literal_expr");
        Ok(())
    }

    fn visit_module(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_module");
        Ok(())
    }

    fn visit_if_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_if_stmt");
        Ok(())
    }

    fn visit_class_decl(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_class_decl");
        Ok(())
    }

    fn visit_type_decl(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_type_decl");
        Ok(())
    }

    fn visit_variable_decl(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_variable_decl");
        Ok(())
    }

    fn visit_ternary_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_ternary_expr");
        Ok(())
    }

    fn visit_lambda_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_lambda_expr");
        Ok(())
    }

    fn visit_assignment_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_assignment_expr");
        Ok(())
    }

    fn visit_attribute_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_attribute_expr");
        Ok(())
    }

    fn visit_subscription_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_subscription_expr");
        Ok(())
    }

    fn visit_dict_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_dict_expr");
        Ok(())
    }

    fn visit_list(&mut self, _node_ids: &[NodeID]) -> VisitorResult<Vec<()>> {
        self.visited_nodes.push("visit_list");
        Ok(vec![])
    }

    fn visit_tuple_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_tuple_expr");
        Ok(())
    }

    fn visit_argument_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_argument_expr");
        Ok(())
    }

    fn visit_from_import_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_from_import_stmt");
        Ok(())
    }

    fn visit_import_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_import_stmt");
        Ok(())
    }

    fn visit_generic_type(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_generic_type");
        Ok(())
    }

    fn visit_callable_type(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_callable_type");
        Ok(())
    }

    fn visit_union_type(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_union_type");
        Ok(())
    }

    fn visit_basic_ident(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_basic_identifier");
        Ok(())
    }

    fn visit_variable_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_variable_expr");
        Ok(())
    }

    fn visit_parameter_ident(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_parameter_ident");
        Ok(())
    }

    fn visit_assert_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_assert_stmt");
        Ok(())
    }

    fn visit_assignment_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_assignment_stmt");
        Ok(())
    }

    fn visit_expression_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_expression_stmt");
        Ok(())
    }

    fn visit_pass_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_pass_stmt");
        Ok(())
    }

    fn visit_break_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_break_stmt");
        Ok(())
    }

    fn visit_continue_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_continue_stmt");
        Ok(())
    }

    fn visit_for_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_for_stmt");
        Ok(())
    }

    fn visit_return_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_return_stmt");
        Ok(())
    }

    fn visit_while_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_while_stmt");
        Ok(())
    }

    fn visit_try_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_try_stmt");
        Ok(())
    }

    fn visit_raise_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_raise_stmt");
        Ok(())
    }

    fn visit_except_handler(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_except_handler");
        Ok(())
    }

    fn visit_call_expr(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        self.visited_nodes.push("visit_call");
        Ok(())
    }
}

#[test]
fn test_any_node_visitable_binary_op() {
    let mut ast = AST::new();
    let span = Span::new(0, 5);

    // Create a binary op node
    let binary_op = BinaryOpExpr {
        left: NodeID::new(0, 0),
        right: NodeID::new(0, 0),
        op: BinaryOpKind::Add,
        id: NodeID::new(0, 0),
        parent: None,
        span,
    };
    let binary_op_node = AnyNode::BinaryOpExpr(binary_op);
    let binary_op_id = ast.alloc_node(NodeKind::Expression, binary_op_node, span);

    // Create a test visitor
    let mut visitor = TestVisitor::new();

    // Call accept on the AnyNode via AST
    if let Some(node) = ast.get_node(binary_op_id) {
        drop(node.data.accept(&mut visitor, binary_op_id));
    }

    // Verify that the binary_op visitor method was called
    assert_eq!(visitor.visited_nodes, vec!["visit_binary_op"]);
}

#[test]
fn test_any_node_visitable_unary_op() {
    let mut ast = AST::new();
    let span = Span::new(0, 5);

    // Create a unary op node
    let unary_op = UnaryOpExpr {
        operand: NodeID::new(0, 0),
        op: UnaryOpKind::Neg,
        id: NodeID::new(0, 0),
        parent: None,
        span,
    };
    let unary_op_node = AnyNode::UnaryOpExpr(unary_op);
    let unary_op_id = ast.alloc_node(NodeKind::Expression, unary_op_node, span);

    // Create a test visitor
    let mut visitor = TestVisitor::new();

    // Call accept on the AnyNode via AST
    if let Some(node) = ast.get_node(unary_op_id) {
        drop(node.data.accept(&mut visitor, unary_op_id));
    }

    // Verify that the unary_op visitor method was called
    assert_eq!(visitor.visited_nodes, vec!["visit_unary_op"]);
}

#[test]
fn test_any_node_visitable_literal() {
    let mut ast = AST::new();
    let span = Span::new(0, 5);

    // Create a literal node
    let literal = LiteralExpr {
        kind: LiteralValue::Int(0),
        raw_value: "42".to_string(),
        id: NodeID::new(0, 0),
        parent: None,
        span,
    };
    let literal_node = AnyNode::LiteralExpr(literal);
    let literal_id = ast.alloc_node(NodeKind::Expression, literal_node, span);

    // Create a test visitor
    let mut visitor = TestVisitor::new();

    // Call accept on the AnyNode via AST
    if let Some(node) = ast.get_node(literal_id) {
        drop(node.data.accept(&mut visitor, literal_id));
    }

    // Verify that the literal visitor method was called
    assert_eq!(visitor.visited_nodes, vec!["visit_literal"]);
}

#[test]
fn test_any_node_visitable_module() {
    let mut ast = AST::new();
    let span = Span::new(0, 5);

    // Create a module node
    let module = Module::new("test.ty".to_string(), vec![], NodeID::placeholder(), span);
    let module_node = AnyNode::Module(module);
    let module_id = ast.alloc_node(NodeKind::Module, module_node, span);

    // Create a test visitor
    let mut visitor = TestVisitor::new();

    // Call accept on the AnyNode via AST
    if let Some(node) = ast.get_node(module_id) {
        drop(node.data.accept(&mut visitor, module_id));
    }

    // Verify that the module visitor method was called
    assert_eq!(visitor.visited_nodes, vec!["visit_module"]);
}

#[test]
fn test_multiple_node_types_visitable() {
    let mut ast = AST::new();
    let span = Span::new(0, 5);

    // Create several different node types
    let literal = LiteralExpr {
        kind: LiteralValue::String(String::new()),
        raw_value: "\"test\"".to_string(),
        id: NodeID::new(0, 0),
        parent: None,
        span,
    };
    let literal_id = ast.alloc_node(NodeKind::Expression, AnyNode::LiteralExpr(literal), span);
    let module = Module::new("test.ty".to_string(), vec![], NodeID::placeholder(), span);
    let module_id = ast.alloc_node(NodeKind::Module, AnyNode::Module(module), span);
    let binary_op = BinaryOpExpr {
        left: literal_id,
        right: NodeID::new(2, 0),
        op: BinaryOpKind::Add,
        id: NodeID::new(2, 0),
        parent: None,
        span,
    };
    let binary_op_id = ast.alloc_node(NodeKind::Expression, AnyNode::BinaryOpExpr(binary_op), span);

    // Visit each node with our test visitor
    let mut visitor = TestVisitor::new();

    if let Some(node) = ast.get_node(literal_id) {
        drop(node.data.accept(&mut visitor, literal_id));
    }

    if let Some(node) = ast.get_node(module_id) {
        drop(node.data.accept(&mut visitor, module_id));
    }

    if let Some(node) = ast.get_node(binary_op_id) {
        drop(node.data.accept(&mut visitor, binary_op_id));
    }

    // Verify that all visitor methods were called in the right order
    assert_eq!(visitor.visited_nodes, vec!["visit_literal", "visit_module", "visit_binary_op"]);
}

// Test direct concrete type visitation
#[test]
fn test_concrete_type_visitable() {
    let span = Span::new(0, 5);

    // Create a concrete binary_op instance (not wrapped in AnyNode)
    let binary_op = BinaryOpExpr {
        left: NodeID::new(0, 0),
        right: NodeID::new(0, 0),
        op: BinaryOpKind::Add,
        id: NodeID::new(0, 0),
        parent: None,
        span,
    };

    let mut visitor = TestVisitor::new();

    // Call accept directly on the concrete type
    let node_id = NodeID::new(1, 1); // Dummy NodeID for testing
    drop(binary_op.accept(&mut visitor, node_id));

    // Verify that the concrete visitor method was called
    assert_eq!(visitor.visited_nodes, vec!["visit_binary_op"]);
}

// Additional tests for newly implemented node types

// Test for Statement types
#[test]
fn test_statement_types_visitable() {
    let span = Span::new(0, 5);
    let mut visitor = TestVisitor::new();
    let dummy_id = NodeID::placeholder();

    // Create and test Pass statement
    let pass = PassStmt::new(dummy_id, span);
    drop(pass.accept(&mut visitor, dummy_id));

    // Create and test Break statement
    let break_stmt = BreakStmt::new(dummy_id, span);
    drop(break_stmt.accept(&mut visitor, dummy_id));

    // Create and test Continue statement
    let continue_stmt = ContinueStmt::new(dummy_id, span);
    drop(continue_stmt.accept(&mut visitor, dummy_id));

    // Verify that the visitor methods were called
    assert_eq!(visitor.visited_nodes, vec!["visit_pass", "visit_break", "visit_continue"]);
}

// Test for Expression types
#[test]
fn test_expression_types_visitable() {
    let span = Span::new(0, 5);
    let mut visitor = TestVisitor::new();
    let dummy_id = NodeID::new(1, 1);

    // Create and test Lambda expression
    let lambda = LambdaExpr::new(vec![dummy_id], dummy_id, dummy_id, span);
    drop(lambda.accept(&mut visitor, dummy_id));

    // Verify that the visitor methods were called
    assert_eq!(visitor.visited_nodes, vec!["visit_lambda"]);
}

// Test for Container types
#[test]
fn test_container_types_visitable() {
    let span = Span::new(0, 5);
    let mut visitor = TestVisitor::new();
    let dummy_id = NodeID::new(1, 1);

    // Create and test Dict expression
    let dict = DictExpr::new(vec![(dummy_id, dummy_id)], dummy_id, span);
    drop(dict.accept(&mut visitor, dummy_id));

    // Create and test List expression
    let list = ListExpr::new(vec![dummy_id], dummy_id, span);
    drop(list.accept(&mut visitor, dummy_id));

    // Create and test Tuple expression
    let tuple = TupleExpr::new(vec![dummy_id], dummy_id, span);
    drop(tuple.accept(&mut visitor, dummy_id));

    // Verify that the visitor methods were called
    assert_eq!(visitor.visited_nodes, vec!["visit_dict", "visit_list", "visit_tuple"]);
}

// Test for Module-level constructs
#[test]
fn test_module_constructs_visitable() {
    let span = Span::new(0, 5);
    let mut visitor = TestVisitor::new();
    let dummy_id = NodeID::new(1, 1);

    // Create and test Import
    let import = ImportStmt {
        module_parts: vec!["module".to_string()],
        alias: None,
        id: dummy_id,
        parent: None,
        span,
    };
    drop(import.accept(&mut visitor, dummy_id));

    // Create and test FromImport
    let from_import = FromImportStmt {
        module_parts: vec!["module".to_string()],
        names: vec![("name".to_string(), None)],
        level: 0,
        id: dummy_id,
        parent: None,
        span,
    };
    drop(from_import.accept(&mut visitor, dummy_id));

    // Verify that the visitor methods were called
    assert_eq!(visitor.visited_nodes, vec!["visit_import", "visit_from_import"]);
}

// Test for unified BasicIdent
#[test]
fn test_basic_identifier_visitable() {
    let span = Span::new(0, 5);
    let mut visitor = TestVisitor::new();
    let dummy_id = NodeID::new(1, 1);

    // Create and test BasicIdent (covers all naming conventions)
    let basic_id = BasicIdent::new("identifier".to_string(), dummy_id, span);
    drop(basic_id.accept(&mut visitor, dummy_id));

    // Create a constant-style identifier
    let const_style = BasicIdent::new("CONSTANT".to_string(), dummy_id, span);
    drop(const_style.accept(&mut visitor, dummy_id));

    // Create a private-style identifier
    let private_style = BasicIdent::new("_private".to_string(), dummy_id, span);
    drop(private_style.accept(&mut visitor, dummy_id));

    // Create a mangled-style identifier
    let mangled_style = BasicIdent::new("__mangled".to_string(), dummy_id, span);
    drop(mangled_style.accept(&mut visitor, dummy_id));

    // Verify that all visit the same method
    assert_eq!(
        visitor.visited_nodes,
        vec![
            "visit_basic_identifier",
            "visit_basic_identifier",
            "visit_basic_identifier",
            "visit_basic_identifier"
        ]
    );
}

// Test for type expressions
#[test]
fn test_type_expressions_visitable() {
    let span = Span::new(0, 5);
    let mut visitor = TestVisitor::new();
    let dummy_id = NodeID::new(1, 1);

    // Create and test GenericType
    let generic_type = GenericType::new(dummy_id, vec![dummy_id], dummy_id, span);
    drop(generic_type.accept(&mut visitor, dummy_id));

    // Create and test CallableType
    let callable_type = CallableType::new(vec![dummy_id], dummy_id, dummy_id, span);
    drop(callable_type.accept(&mut visitor, dummy_id));

    // Create and test UnionType
    let union_type = UnionType::new(vec![dummy_id], dummy_id, span);
    drop(union_type.accept(&mut visitor, dummy_id));

    // Verify that the visitor methods were called
    assert_eq!(
        visitor.visited_nodes,
        vec!["visit_generic_type", "visit_callable_type", "visit_union_type"]
    );
}
