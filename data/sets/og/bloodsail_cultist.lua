local card = {
    api_version = 1, id = "OG_315", name = "Bloodsail Cultist",
    text = "<b>Battlecry:</b> If you control another Pirate, give your weapon +1/+1.", set = "OG",
    type = "minion", class = "warrior", rarity = "rare", cost = 3, attack = 3, health = 4,
    tags = { "pirate" }, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local weapon = ctx:player(player).weapon
    if weapon == nil then return end
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self then
            local definition = ctx:card_definition(ctx:entity(minion).card_id)
            for _, tag in ipairs(definition.tags or {}) do
                if tag == "pirate" or tag == "all" then cardlib.effects.buff(ctx, weapon, 1, 1) return end
            end
        end
    end
end
return card
