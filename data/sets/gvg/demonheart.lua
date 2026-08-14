local function is_demon(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "demon" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_019",
    name = "Demonheart",
    text = "Deal $5 damage to a minion.  If it's a friendly Demon, give it +5/+5 instead.",
    set = "GVG",
    type = "spell",
    class = "warlock",
    rarity = "epic",
    spell_school = "shadow",
    cost = 5,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        if ctx:controller(target) == ctx:controller(self) and is_demon(ctx, target) then
            ctx:buff(target, 5, 5)
        else
            ctx:damage(target, 5)
        end
    end,
}
