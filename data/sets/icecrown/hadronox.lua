local function had_keyword(record, wanted)
    for _, keyword in ipairs(record.keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1, id = "ICC_835", name = "Hadronox",
    text = "<b>Deathrattle:</b> Summon your <b>Taunt</b> minions that\ndied this game.",
    set = "ICECROWN", type = "minion", class = "druid", rarity = "legendary",
    cost = 9, attack = 3, health = 7, tags = { "beast" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local player = ctx:controller(self)
        for _, record in ipairs(ctx:minion_death_records(player)) do
            if had_keyword(record, "taunt") then ctx:summon(player, record.card_id) end
        end
    end,
}
