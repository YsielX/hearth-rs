local function is_beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "LOE_073",
    name = "Fossilized Devilsaur",
    text = "<b>Battlecry:</b> If you control another Beast, gain <b>Taunt</b>.",
    set = "LOE",
    type = "minion",
    rarity = "common",
    cost = 8,
    attack = 8,
    health = 8,
    tags = { "undead", "beast" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self and is_beast(ctx, minion) then
                cardlib.effects.grant_keyword(ctx, self, "taunt")
                return
            end
        end
    end,
}
