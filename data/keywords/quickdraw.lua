return {
    api_version = 1, module_type = "keyword", id = "quickdraw", name = "Quickdraw",
    required_card_hooks = { "on_quickdraw" },
    hooks = { on_play = function(ctx, self, target)
        if not ctx:entered_hand_this_turn(self) then return end
        if target == nil then ctx:continue_with("on_quickdraw")
        else ctx:continue_with_entity("on_quickdraw", target) end
    end },
}
