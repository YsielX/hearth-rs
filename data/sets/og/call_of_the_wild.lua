local card = {
    api_version = 1, id = "OG_211", name = "Call of the Wild",
    text = "Summon all three Animal Companions.", set = "OG", type = "spell",
    class = "hunter", rarity = "epic", cost = 8,
    rules = {
        can_play = function(ctx, self, current)
            return current and #ctx:board(ctx:controller(self)) < 7
        end,
    },
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:summon(player, "NEW1_034")
        ctx:summon(player, "NEW1_033")
        ctx:summon(player, "NEW1_032")
    end,
}
card.tokens = {
    { id = "NEW1_034", name = "Huffer", text = "<b>Charge</b>", set = "LEGACY",
      type = "minion", class = "hunter", cost = 3, attack = 4, health = 2,
      tags = { "beast" }, keywords = { "charge" } },
    { id = "NEW1_033", name = "Leokk", text = "Your other minions have +1 Attack.",
      set = "LEGACY", type = "minion", class = "hunter", cost = 3, attack = 2,
      health = 4, tags = { "beast" }, auras = {{ attack = 1,
        targets = function(ctx, self)
            local result = {}
            for _, minion in ipairs(ctx:friendly_minions(self)) do
                local dormant = false
                for _, keyword in ipairs(ctx:entity(minion).keywords) do
                    if keyword == "dormant" then dormant = true break end
                end
                if minion ~= self and not dormant then result[#result + 1] = minion end
            end
            return result
        end }} },
    { id = "NEW1_032", name = "Misha", text = "<b>Taunt</b>", set = "LEGACY",
      type = "minion", class = "hunter", cost = 3, attack = 4, health = 4,
      tags = { "beast" }, keywords = { "taunt" } },
}
return card
