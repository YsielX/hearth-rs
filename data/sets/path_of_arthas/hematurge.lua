local card = {
    api_version = 1,
    id = "RLK_066",
    name = "Hematurge",
    text = "<b>Battlecry:</b> Spend a\n<b>Corpse</b> to <b>Discover</b> a\nBlood Rune card.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "rare",
    cost = 2,
    attack = 2,
    health = 3,
    rune_cost = { blood = 1 },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:spend_resource_and_continue(ctx:controller(self), "corpses", 1, 1, "discover_blood_card")
end

function card.discover_blood_card(ctx, self, spent)
    if spent == 0 then return end
    local player = ctx:controller(self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.class == "death_knight" and definition.rune_cost.blood > 0 then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then
        ctx:discover_cards(
            player,
            ctx:localize(
                "Discover a Blood Rune card",
                "发现一张鲜血符文牌",
                "發現一張血魄符文牌"
            ),
            pool,
            3,
            "receive_blood_card"
        )
    end
end

function card.receive_blood_card(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
