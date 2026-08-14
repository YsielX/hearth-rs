local card = {
    api_version = 1,
    id = "EDR_871",
    name = "Spirit Gatherer",
    text = "<b>Battlecry:</b> Get a Wisp. <b>Imbue</b> your Hero Power.",
    set = "EMERALD_DREAM",
    type = "minion",
    class = "mage",
    cost = 2,
    attack = 2,
    health = 1,
    keywords = { "battlecry", "imbue" },
}

function card.on_battlecry(ctx, self)
    ctx:give_card(ctx:controller(self), "CS2_231")
end

function card.on_imbue(ctx, self)
    local player = ctx:controller(self)
    if ctx:player(player).class == "mage" then
        ctx:replace_hero_power(player, "EDR_851p")
    end
end

card.tokens = {
    { id = "CS2_231", name = "Wisp", text = "", set = "EXPERT1", type = "minion", collectible = true, cost = 0, attack = 1, health = 1, tags = { "undead" } },
    {
        id = "EDR_851p", name = "Blessing of the Wisp",
        text = "[x]Summon <b>@</b> Wisp.\nDeal <b>$@</b> damage\nrandomly split among\nall enemies.",
        set = "EMERALD_DREAM", type = "hero_power", class = "mage", cost = 2,
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            local amount = math.max(1, ctx:get_player_data(player, "imbue_count"))
            for _ = 1, amount do ctx:summon(player, "CS2_231") end
            for _ = 1, amount do ctx:random_entity(ctx:enemy_characters(self), "deal_wisp_damage") end
        end,
        deal_wisp_damage = function(ctx, self, target) ctx:damage(target, 1) end,
    },
}

return card
