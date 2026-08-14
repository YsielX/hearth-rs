return {
    api_version = 1, module_type = "keyword", id = "adapt", name = "Adapt",
    required_card_hooks = { "on_adapt" },
    hooks = { on_play = function(ctx, self, target)
        if target == nil then ctx:continue_with("on_adapt")
        else ctx:continue_with_entity("on_adapt", target) end
    end },
}
