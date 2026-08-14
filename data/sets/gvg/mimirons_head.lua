local card = {
    api_version = 1, id = "GVG_111", name = "Mimiron's Head",
    text = "At the start of your turn, if you have at least 3 Mechs, destroy them all and form V-07-TR-0N.",
    set = "GVG", type = "minion", rarity = "legendary", cost = 5, attack = 4, health = 5,
    tags = { "mech" },
    tokens = {{ id = "GVG_111t", name = "V-07-TR-0N", text = "<b>Charge</b>\n<b>Mega-Windfury</b>", set = "GVG", type = "minion", cost = 8, attack = 4, health = 8, tags = { "mech" }, keywords = { "charge", "mega_windfury" } }},
}
local function friendly_mechs(ctx, self)
    local mechs = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        for _, tag in ipairs(ctx:card_definition(ctx:entity(minion).card_id).tags) do
            if tag == "mech" then mechs[#mechs + 1] = minion break end
        end
    end
    return mechs
end
card.triggers = {{
    event = "turn_started", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and #friendly_mechs(ctx, self) >= 3
    end,
    effect = function(ctx, self)
        ctx:destroy_all(friendly_mechs(ctx, self))
        ctx:continue_with("form_voltron")
    end,
}}
function card.form_voltron(ctx, self) ctx:summon(ctx:controller(self), "GVG_111t") end
return card
