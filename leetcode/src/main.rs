// Definition for singly-linked list.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

fn merge_two_lists(l1: Option<Box<ListNode>>, l2: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    if l1 == None {
        return l2
    }
    if l2 == None {
        return l1
    }

    let mut l1 = l1;
    let mut l2 = l2;
    
    let mut head = Some(Box::new(ListNode::new(-1)));

    // Option implements Deref trait, and so does Box,
    // so Deref coercion is happening here...
    let mut result = head.as_deref_mut().unwrap();

    while l1 != None && l2 != None {
        if l1.as_ref().unwrap().val < l2.as_ref().unwrap().val {
            result.next = l1.clone();
            l1 = l1.as_ref().unwrap().next.clone();
        }
        else {
            result.next = l2.clone();
            l2 = l2.as_ref().unwrap().next.clone();
        }
        result = result.next.as_deref_mut().unwrap();
    }

    if l1 != None {
        result.next = l1;
    } else if l2 != None {
        result.next = l2;
    }

    head.as_ref().unwrap().next.clone()
}

pub fn add_two_numbers(l1: Option<Box<ListNode>>, l2: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    let mut carry = 0;
    let mut result = Some(Box::new(ListNode::new(-1)));
    let mut result_ref = result.as_mut().unwrap();

    let (mut l1, mut l2) = (l1, l2);

    while l1 != None && l2 != None {
        let value = l1.as_ref().unwrap().val + l2.as_ref().unwrap().val + carry;
        result_ref.next = Some(Box::new(ListNode::new(value % 10)));
        carry = value / 10;
        l1 = l1.unwrap().next;
        l2 = l2.unwrap().next;
        result_ref = result_ref.next.as_mut().unwrap();
    }

    if l2 != None {
        l1 = l2
    }

    while l1 != None {
        let value = l1.as_ref().unwrap().val + carry;
        result_ref.next = Some(Box::new(ListNode::new(value % 10)));
        carry = value / 10;
        l1 = l1.unwrap().next;
        result_ref = result_ref.next.as_mut().unwrap();
    }

    if carry > 0 {
        result_ref.next = Some(Box::new(ListNode::new(1)));
    }

    result.unwrap().next
}

fn main() {
    
}
