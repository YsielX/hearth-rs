return {
    api_version = 1,
    module_type = "keyword",
    id = "combo",
    name = "Combo",

    required_card_hooks = { "on_combo" },
    hooks = {
        on_play = function(ctx, self, target)
            if not ctx:combo_active(self) then
                return
            end
            if target == nil then
                ctx:continue_with("on_combo")
            else
                ctx:continue_with_entity("on_combo", target)
            end
        end,
    },
}
