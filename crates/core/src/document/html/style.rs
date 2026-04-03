use super::css::{AttributeOperator, Combinator, PseudoClass};
use super::css::{CssParser, Rule, Selector, SimpleSelector};
use super::dom::NodeRef;
use fxhash::FxHashMap;

pub type PropertyMap = FxHashMap<String, String>;

#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        StyleSheet { rules: Vec::new() }
    }

    pub fn append(&mut self, other: &mut Self, sort: bool) {
        if sort {
            other.sort();
        }
        self.rules.append(&mut other.rules);
    }

    pub fn sort(&mut self) {
        self.rules
            .sort_by_cached_key(|rule| rule.selector.specificity());
    }
}

pub fn specified_values(node: NodeRef, stylesheet: &StyleSheet) -> PropertyMap {
    let mut props = FxHashMap::default();
    let mut important = Vec::new();

    for rule in stylesheet
        .rules
        .iter()
        .filter(|rule| rule.selector.matches(node))
    {
        for declaration in &rule.declarations {
            if declaration.important {
                important.push([&declaration.name, &declaration.value]);
            } else {
                expand_and_insert(&declaration.name, &declaration.value, &mut props);
            }
        }
    }

    let local_declarations = node
        .attribute("style")
        .map(|text| CssParser::new(text).parse_declarations())
        .unwrap_or_default();

    for declaration in &local_declarations {
        expand_and_insert(&declaration.name, &declaration.value, &mut props);
    }

    for [name, value] in important {
        expand_and_insert(name, value, &mut props);
    }

    props
}

impl Selector {
    fn matches(&self, node: NodeRef) -> bool {
        let index = self.simple_selectors.len().saturating_sub(1);
        self.matches_rec(node, index)
    }

    fn matches_rec(&self, node: NodeRef, index: usize) -> bool {
        let comb = self.combinators[index];
        let selec = &self.simple_selectors[index];

        if !selec.matches(node) {
            return false;
        }

        match comb {
            Combinator::Child => {
                if let Some(parent) = node.parent_element() {
                    if self.matches_rec(parent, index - 1) {
                        return true;
                    }
                }
                false
            }
            Combinator::Descendant => {
                for anc in node.ancestor_elements() {
                    if self.matches_rec(anc, index - 1) {
                        return true;
                    }
                }
                false
            }
            Combinator::NextSibling => {
                if let Some(nsib) = node.previous_sibling_element() {
                    if self.matches_rec(nsib, index - 1) {
                        return true;
                    }
                }
                false
            }
            Combinator::SubsequentSibling => {
                for sib in node.previous_sibling_elements() {
                    if self.matches_rec(sib, index - 1) {
                        return true;
                    }
                }
                false
            }
            Combinator::None => true,
        }
    }
}

impl SimpleSelector {
    fn matches(&self, node: NodeRef) -> bool {
        if self
            .tag_name
            .iter()
            .any(|name| node.tag_name() != Some(name))
        {
            return false;
        }

        if self
            .classes
            .iter()
            .any(|class| node.classes().all(|c| c != class))
        {
            return false;
        }

        if self.id.iter().any(|id| node.id() != Some(id)) {
            return false;
        }

        if self.attributes.iter().any(|attr| {
            node.attribute(&attr.name)
                .map(|value| attr.operator.matches(value))
                != Some(true)
        }) {
            return false;
        }

        if self.pseudo_classes.iter().any(|pc| !pc.matches(node)) {
            return false;
        }

        true
    }
}

impl AttributeOperator {
    fn matches(&self, value: &str) -> bool {
        match self {
            AttributeOperator::Exists => true,
            AttributeOperator::Matches(v) => v == value,
            AttributeOperator::Contains(v) => value.split_whitespace().any(|value| value == v),
            AttributeOperator::StartsWith(v) => {
                v == value || (value.starts_with(v) && value[v.len()..].starts_with('-'))
            }
        }
    }
}

impl PseudoClass {
    fn matches(&self, node: NodeRef) -> bool {
        match self {
            PseudoClass::FirstChild => node.previous_sibling_element().is_none(),
            PseudoClass::LastChild => node.next_sibling_element().is_none(),
            PseudoClass::FirstOfType => {
                let tag = node.tag_name();
                node.previous_sibling_element()
                    .map(|s| s.tag_name() != tag)
                    .unwrap_or(true)
            }
            PseudoClass::LastOfType => {
                let tag = node.tag_name();
                node.next_sibling_element()
                    .map(|s| s.tag_name() != tag)
                    .unwrap_or(true)
            }
            PseudoClass::NthChild(n) => count_previous_siblings(node) == *n,
            PseudoClass::NthLastChild(n) => count_next_siblings(node) == *n,
            PseudoClass::OnlyChild => {
                node.previous_sibling_element().is_none() && node.next_sibling_element().is_none()
            }
            PseudoClass::Empty => node.first_child().is_none(),
        }
    }
}

fn count_previous_siblings(node: NodeRef) -> i32 {
    let mut count = 0;
    let mut sibling = node.previous_sibling_element();
    while sibling.is_some() {
        count += 1;
        sibling = sibling.and_then(|s| s.previous_sibling_element());
    }
    count
}

fn count_next_siblings(node: NodeRef) -> i32 {
    let mut count = 0;
    let mut sibling = node.next_sibling_element();
    while sibling.is_some() {
        count += 1;
        sibling = sibling.and_then(|s| s.next_sibling_element());
    }
    count
}

fn expand_and_insert(name: &str, value: &str, props: &mut PropertyMap) {
    match name {
        "margin" | "padding" => {
            let values = value.split_whitespace().collect::<Vec<&str>>();
            match values.len() {
                1 => {
                    props.insert(format!("{}-top", name), value.to_string());
                    props.insert(format!("{}-right", name), value.to_string());
                    props.insert(format!("{}-bottom", name), value.to_string());
                    props.insert(format!("{}-left", name), value.to_string());
                }
                2 => {
                    let vertical = values[0];
                    let horizontal = values[1];
                    props.insert(format!("{}-top", name), vertical.to_string());
                    props.insert(format!("{}-right", name), horizontal.to_string());
                    props.insert(format!("{}-bottom", name), vertical.to_string());
                    props.insert(format!("{}-left", name), horizontal.to_string());
                }
                3 => {
                    let top = values[0];
                    let horizontal = values[1];
                    let bottom = values[2];
                    props.insert(format!("{}-top", name), top.to_string());
                    props.insert(format!("{}-right", name), horizontal.to_string());
                    props.insert(format!("{}-bottom", name), bottom.to_string());
                    props.insert(format!("{}-left", name), horizontal.to_string());
                }
                4 => {
                    let top = values[0];
                    let right = values[1];
                    let bottom = values[2];
                    let left = values[3];
                    props.insert(format!("{}-top", name), top.to_string());
                    props.insert(format!("{}-right", name), right.to_string());
                    props.insert(format!("{}-bottom", name), bottom.to_string());
                    props.insert(format!("{}-left", name), left.to_string());
                }
                _ => (),
            }
        }
        "border" => {
            props.insert("border-top".to_string(), value.to_string());
            props.insert("border-right".to_string(), value.to_string());
            props.insert("border-bottom".to_string(), value.to_string());
            props.insert("border-left".to_string(), value.to_string());
        }
        "border-width" | "border-style" | "border-color" => {
            let values = value.split_whitespace().collect::<Vec<&str>>();
            let suffix = name.strip_prefix("border-").unwrap_or("");
            match values.len() {
                1 => {
                    props.insert(format!("border-top-{}", suffix), value.to_string());
                    props.insert(format!("border-right-{}", suffix), value.to_string());
                    props.insert(format!("border-bottom-{}", suffix), value.to_string());
                    props.insert(format!("border-left-{}", suffix), value.to_string());
                }
                2 => {
                    let vertical = values[0];
                    let horizontal = values[1];
                    props.insert(format!("border-top-{}", suffix), vertical.to_string());
                    props.insert(format!("border-right-{}", suffix), horizontal.to_string());
                    props.insert(format!("border-bottom-{}", suffix), vertical.to_string());
                    props.insert(format!("border-left-{}", suffix), horizontal.to_string());
                }
                3 => {
                    let top = values[0];
                    let horizontal = values[1];
                    let bottom = values[2];
                    props.insert(format!("border-top-{}", suffix), top.to_string());
                    props.insert(format!("border-right-{}", suffix), horizontal.to_string());
                    props.insert(format!("border-bottom-{}", suffix), bottom.to_string());
                    props.insert(format!("border-left-{}", suffix), horizontal.to_string());
                }
                4 => {
                    let top = values[0];
                    let right = values[1];
                    let bottom = values[2];
                    let left = values[3];
                    props.insert(format!("border-top-{}", suffix), top.to_string());
                    props.insert(format!("border-right-{}", suffix), right.to_string());
                    props.insert(format!("border-bottom-{}", suffix), bottom.to_string());
                    props.insert(format!("border-left-{}", suffix), left.to_string());
                }
                _ => (),
            }
        }
        "border-top" | "border-right" | "border-bottom" | "border-left" => {
            let side = name.strip_prefix("border-").unwrap_or("");
            props.insert(format!("border-{}-width", side), value.to_string());
            props.insert(format!("border-{}-style", side), value.to_string());
            props.insert(format!("border-{}-color", side), value.to_string());
        }
        _ => {
            props.insert(name.to_string(), value.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::css::CssParser;
    use super::super::xml::XmlParser;
    use super::specified_values;

    #[test]
    fn simple_style() {
        let xml1 = XmlParser::new("<a class='c x y' style='c: 7'/>").parse();
        let xml2 = XmlParser::new("<a id='e' class='x y'/>").parse();
        let mut css = CssParser::new(
            "a { b: 23 }\
                                      .c.x.y { b: 6; c: 3 }\
                                      #e { b: 5 }\
                                      .y { b: 2 }",
        )
        .parse();
        css.sort();
        let n1 = xml1.root().first_child().expect("xml root has no children");
        let n2 = xml2.root().first_child().expect("xml root has no children");
        assert_eq!(
            specified_values(n1, &css),
            [
                ("b".to_string(), "6".to_string()),
                ("c".to_string(), "7".to_string())
            ]
            .iter()
            .cloned()
            .collect()
        );
        assert_eq!(
            specified_values(n2, &css),
            [("b".to_string(), "5".to_string())]
                .iter()
                .cloned()
                .collect()
        );
    }
}
