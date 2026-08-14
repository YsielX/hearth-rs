return {
    api_version = 1, module_type = "keyword", id = "kindred", name = "Kindred",
    required_card_hooks = { "on_kindred" },
    hooks = { on_play = function(ctx, self, target)
        local mine = ctx:card_definition(ctx:entity(self).card_id)
        local active = false
        for _, card_id in ipairs(ctx:cards_played_last_turn(ctx:controller(self))) do
            local previous = ctx:card_definition(card_id)
            if mine.spell_school ~= nil and mine.spell_school == previous.spell_school then
                active = true
            end
            for _, my_tag in ipairs(mine.tags) do
                for _, old_tag in ipairs(previous.tags) do
                    if my_tag == old_tag then active = true end
                end
            end
        end
        if not active then return end
        if target == nil then ctx:continue_with("on_kindred")
        else ctx:continue_with_entity("on_kindred", target) end
    end },
}
