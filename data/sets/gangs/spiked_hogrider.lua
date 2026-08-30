local card = {
    api_version = 1, id = "CFM_688", name = "Spiked Hogrider",
    text = "<b>Battlecry:</b> If an enemy minion has <b>Taunt</b>, gain <b>Charge</b>.",
    set = "GANGS", type = "minion", rarity = "rare", cost = 5, attack = 5,
    health = 5, tags = { "quilboar" }, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        for _, keyword in ipairs(ctx:entity(minion).keywords or {}) do
            if keyword == "taunt" then cardlib.effects.grant_keyword(ctx, self, "charge") return end
        end
    end
end
return card
