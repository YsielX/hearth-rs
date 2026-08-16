local card = {
    api_version = 1, id = "GVG_108", name = "Recombobulator",
    text = "<b>Battlecry:</b> Transform a friendly minion into a random minion with the same Cost.",
    set = "GVG", type = "minion", rarity = "epic", cost = 2, attack = 3, health = 2,
    keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self then result[#result + 1] = minion end
        end
        return result
    end,
}
function card.on_battlecry(ctx, self, target)
    if target == nil then return end
    local cost = ctx:entity(target).cost
    local choices = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.type == "minion" and definition.cost == cost then choices[#choices + 1] = id end
    end
    if #choices > 0 then
        ctx:set_data(self, "recombobulate_target", target)
        ctx:random_value(choices, "transform_target")
    end
end
function card.transform_target(ctx, self, id)
    cardlib.effects.transform(ctx, ctx:get_data(self, "recombobulate_target"), id)
end
return card
